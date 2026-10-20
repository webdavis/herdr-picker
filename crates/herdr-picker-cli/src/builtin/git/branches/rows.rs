use herdr_picker_domain::{Action, Key, PickerRow, Preview};
use herdr_picker_protocol::shell_word;
use serde_json::json;

use super::super::paint::{blue, dim, green, mauve, yellow};
use super::super::status::words;
use super::super::steps::{confirm, next};
use super::super::worktrunk::{Item, Worktree};
use super::super::{Repo, wt};
use super::Ref;
use crate::builtin::rows::key;

pub(super) fn local(repo: &Repo, item: &Item, current: bool) -> PickerRow {
    let name = item.branch.clone().unwrap_or_default();
    let elsewhere = item
        .worktree
        .as_ref()
        .filter(|tree| tree.path != repo.root.to_string_lossy());
    let mut label = green(&name);
    if current {
        label += &format!(" {}", yellow("current"));
    }
    for part in [&item.status, &dim(&item.subject)] {
        if !part.is_empty() {
            label += &format!("  {part}");
        }
    }
    if let Some(tree) = elsewhere {
        label += &format!("  {}", dim(&format!("in {}", tree.path)));
    }
    let checkout = match elsewhere {
        Some(tree) => Action::Cd(tree.path.clone()),
        None => next("checkout", &["local", &name], None, None),
    };
    let mut keys = vec![
        (key("ctrl-enter"), checkout),
        (
            key("ctrl-t"),
            confirm(
                &format!("Open {name} in its own worktree?"),
                wt(&repo.root, &["switch", &name]),
            ),
        ),
        (
            key("ctrl-e"),
            next("rename-branch", &[&name], Some(&name), None),
        ),
    ];
    if let Some(tree) = item.worktree.as_ref().filter(|tree| !tree.main) {
        keys.push((key("ctrl-x"), remove(repo, tree, Some(&name))));
    }
    row(
        repo,
        label,
        &name,
        &format!("refs/heads/{name}"),
        keys,
        "local branches",
    )
}

pub(super) fn shared(repo: &Repo, found: &Ref) -> PickerRow {
    let (painted, kind, group) = match found.area.as_str() {
        "remotes" => (blue(&found.name), "remote", "remote branches"),
        _ => (mauve(&found.name), "tag", "tags"),
    };
    let label = format!("{painted}  {}", dim(&found.subject));
    let checkout = next("checkout", &[kind, &found.name], None, None);
    let full = match kind {
        "remote" => format!("refs/remotes/{}", found.name),
        _ => format!("refs/tags/{}", found.name),
    };
    row(
        repo,
        label,
        &found.name,
        &full,
        vec![(key("ctrl-enter"), checkout)],
        group,
    )
}

pub(super) fn worktree(repo: &Repo, tree: &Worktree, branch: Option<&str>) -> PickerRow {
    let state = json!({ "worktree": tree.path }).to_string();
    let mut keys = vec![
        (key("ctrl-enter"), Action::Cd(tree.path.clone())),
        (key("ctrl-s"), next("contents", &[], None, Some(state))),
    ];
    if !tree.main {
        keys.push((key("ctrl-x"), remove(repo, tree, branch)));
    }
    keys.extend(repository_keys(repo));
    PickerRow {
        label: format!(
            "{}  {}",
            blue(&tree.path),
            dim(branch.unwrap_or("detached"))
        ),
        preview: Some(Preview::Path(tree.path.clone())),
        enter: Action::Insert(shell_word(&tree.path)),
        keys,
        group: Some("worktrees".to_string()),
    }
}

fn row(
    repo: &Repo,
    label: String,
    name: &str,
    full: &str,
    mut keys: Vec<(Key, Action)>,
    group: &str,
) -> PickerRow {
    let mut preview = repo.command(&["show", "--color=always", "--no-ext-diff", "--no-textconv"]);
    preview.args(["--stat", "--patch", full]);
    keys.push((key("alt-i"), Action::Run(repo.shell(&["show", full]))));
    keys.extend(repository_keys(repo));
    PickerRow {
        label,
        preview: Some(Preview::Command(words(&preview))),
        enter: Action::Insert(name.to_string()),
        keys,
        group: Some(group.to_string()),
    }
}

// Fetch, pull and prune act on the repository, whichever row is highlighted.
fn repository_keys(repo: &Repo) -> Vec<(Key, Action)> {
    let root = repo.root.to_string_lossy().into_owned();
    vec![
        (
            key("ctrl-f"),
            Action::Run(repo.shell(&["fetch", "--all", "--prune"])),
        ),
        (key("ctrl-l"), next("pull", &[], None, None)),
        (key("alt-d"), next("prune", &[&root], None, None)),
    ]
}

fn remove(repo: &Repo, tree: &Worktree, branch: Option<&str>) -> Action {
    let what = branch.map_or_else(
        || tree.path.clone(),
        |branch| format!("{branch} at {}", tree.path),
    );
    confirm(
        &format!("Remove the worktree of {what}?"),
        wt(&repo.root, &["remove", branch.unwrap_or(&tree.path)]),
    )
}
