use std::io::Write;
use std::thread;

use herdr_picker_domain::{Action, PickerRow};

use super::Repo;
use super::steps::{ask, cancel, choice};
use super::worktrunk::{self};
use crate::builtin::rows::RowWriter;
use crate::builtin::{Places, written};
use crate::command::{CommandError, failed};

mod fallback;
mod rows;

use fallback::from_git;

const REFS: &str = "--format=%(refname)%09%(refname:short)%09%(HEAD)%09%(upstream:short)%09%(upstream:track)%09%(subject)";

// A branch, remote branch or tag as git lists it.
pub(super) struct Ref {
    area: String,
    name: String,
    current: bool,
    tracking: String,
    subject: String,
}

// Local branches, remote branches, tags and the other worktrees of this repository, or the
// repositories with something to prune.
pub(super) fn branches(
    source: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    if source == "repos" {
        return super::prune::repos(&repo, places, rows);
    }
    let root = repo.root.clone();
    // worktrunk takes a moment, so git lists the refs meanwhile.
    let listing = thread::spawn(move || worktrunk::list(&root));
    let refs = refs(&repo)?;
    let items = match listing.join().ok().flatten() {
        Some(items) => items,
        None => from_git(&repo, &refs)?,
    };
    // In git's order, as the branches picker always listed them.
    let mut listed: Vec<PickerRow> = refs
        .iter()
        .filter(|found| found.area == "heads")
        .filter_map(|found| {
            let item = items
                .iter()
                .find(|item| item.branch.as_ref() == Some(&found.name))?;
            Some(rows::local(&repo, item, found.current))
        })
        .collect();
    for area in ["remotes", "tags"] {
        let found = refs.iter().filter(|found| found.area == area);
        listed.extend(found.map(|found| rows::shared(&repo, found)));
    }
    let others = items.iter().filter_map(|item| {
        let tree = item.worktree.as_ref()?;
        (tree.path != repo.root.to_string_lossy())
            .then(|| rows::worktree(&repo, tree, item.branch.as_deref()))
    });
    listed.extend(others);
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

fn refs(repo: &Repo) -> Result<Vec<Ref>, CommandError> {
    let listed = repo.git(&[
        "for-each-ref",
        REFS,
        "refs/heads",
        "refs/remotes",
        "refs/tags",
    ])?;
    Ok(String::from_utf8_lossy(&listed)
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.splitn(6, '\t').collect();
            let [reference, name, head, upstream, track, subject] = fields[..] else {
                return None;
            };
            (!reference.ends_with("/HEAD")).then(|| Ref {
                area: reference.split('/').nth(1).unwrap_or_default().to_string(),
                name: name.to_string(),
                current: head == "*",
                tracking: format!("{upstream} {track}").trim().to_string(),
                subject: subject.to_string(),
            })
        })
        .collect())
}

// Ctrl-P's step: pull into the current branch, merging or rebasing.
pub(super) fn pull(places: &Places, rows: &mut RowWriter<impl Write>) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    let name = String::from_utf8_lossy(&repo.git(&["branch", "--show-current"])?)
        .trim()
        .to_string();
    if name.is_empty() {
        return Err(failed(
            "HEAD is detached, so there is no current branch to pull into.",
        ));
    }
    let choices = vec![
        choice(
            "Pull",
            "merges as your Git settings say",
            Action::Run(repo.shell(&["pull"])),
        ),
        choice(
            "Pull with rebase",
            "replays your commits on top",
            Action::Run(repo.shell(&["pull", "--rebase"])),
        ),
        cancel(),
    ];
    let question = format!("Pull into {name}?");
    ask(&question, &question, choices, rows)
}

#[cfg(test)]
mod tests;
