use std::path::Path;
use std::process::Command;

use serde_json::Value;

use super::output;

// One branch as worktrunk lists it: its status column, and the worktree holding it, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Item {
    pub(super) branch: Option<String>,
    pub(super) subject: String,
    // worktrunk's status, without the branch name its status line starts with.
    pub(super) status: String,
    pub(super) worktree: Option<Worktree>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Worktree {
    pub(super) path: String,
    pub(super) main: bool,
    pub(super) current: bool,
}

// Local branches and worktrees from one `wt list`, so the two never disagree; None without wt.
pub(super) fn list(root: &Path) -> Option<Vec<Item>> {
    let listed = output(Command::new("wt").arg("-C").arg(root).args([
        "list",
        "--branches",
        "--format=json",
    ]))
    .ok()?;
    parse(&String::from_utf8_lossy(&listed))
}

pub(super) fn parse(listed: &str) -> Option<Vec<Item>> {
    let listed: Value = serde_json::from_str(listed).ok()?;
    let items = listed["items"].as_array()?;
    Some(
        items
            .iter()
            .map(|item| {
                let text = |value: &Value| value.as_str().unwrap_or_default().to_string();
                let branch = item["branch"].as_str().map(str::to_string);
                let line = text(&item["display"]["statusline"]);
                let status = branch
                    .as_deref()
                    .and_then(|branch| line.strip_prefix(branch))
                    .unwrap_or(&line)
                    .trim()
                    .to_string();
                let tree = &item["worktree"];
                Item {
                    branch,
                    subject: text(&item["head"]["subject"]),
                    status,
                    worktree: tree["path"].as_str().map(|path| Worktree {
                        path: path.to_string(),
                        main: tree["main"].as_bool().unwrap_or_default(),
                        current: tree["current"].as_bool().unwrap_or_default(),
                    }),
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests;
