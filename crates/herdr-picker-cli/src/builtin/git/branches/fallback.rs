use super::super::worktrunk::{Item, Worktree};
use super::super::{Repo, strings};
use super::Ref;
use crate::command::CommandError;

// Without worktrunk, git says which worktree holds each branch, and its upstream stands in
// for worktrunk's status.
pub(super) fn from_git(repo: &Repo, refs: &[Ref]) -> Result<Vec<Item>, CommandError> {
    let mut trees: Vec<(Worktree, Option<String>)> = Vec::new();
    for record in strings(&repo.git(&["worktree", "list", "--porcelain", "-z"])?) {
        if let Some(path) = record.strip_prefix("worktree ") {
            let main = trees.is_empty();
            trees.push((
                Worktree {
                    path: path.to_string(),
                    main,
                    current: false,
                },
                None,
            ));
        } else if let (Some(branch), Some(last)) =
            (record.strip_prefix("branch refs/heads/"), trees.last_mut())
        {
            last.1 = Some(branch.to_string());
        }
    }
    let held = |name: &str| {
        trees
            .iter()
            .find(|(_, branch)| branch.as_deref() == Some(name))
            .map(|(tree, _)| tree.clone())
    };
    let mut items: Vec<Item> = refs
        .iter()
        .filter(|found| found.area == "heads")
        .map(|found| Item {
            branch: Some(found.name.clone()),
            subject: found.subject.clone(),
            status: found.tracking.clone(),
            worktree: held(&found.name),
        })
        .collect();
    let detached = trees.iter().filter(|(_, branch)| branch.is_none());
    items.extend(detached.map(|(tree, _)| Item {
        branch: None,
        subject: String::new(),
        status: String::new(),
        worktree: Some(tree.clone()),
    }));
    Ok(items)
}
