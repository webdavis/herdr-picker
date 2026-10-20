use std::io::Write;

use herdr_picker_domain::Action;

use super::steps::{ask, cancel, choice};
use super::{Repo, plural, strings};
use crate::builtin::Places;
use crate::builtin::rows::RowWriter;
use crate::command::CommandError;

mod target;

use target::target;

// The step before checking out a branch, tag or commit. With local changes it says how many,
// which of them Git would refuse to carry across, and offers to stash them first.
pub(super) fn checkout(
    kind: &str,
    name: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    let (target, shown, detached) = target(&repo, kind, name)?;
    let what = format!("{shown}{}", if detached { " (detached HEAD)" } else { "" });
    let destination = target.last().cloned().unwrap_or_default();
    let names = |arguments: &[&str]| repo.git(arguments).map(|found| strings(&found));
    let changed = names(&["diff", "HEAD", "--name-only", "-z"])?;
    let untracked = names(&["ls-files", "-z", "--others", "--exclude-standard"])?;
    let differing = names(&["diff", "--name-only", "-z", "HEAD", &destination]).unwrap_or_default();
    let mut blocking: Vec<String> = changed
        .iter()
        .filter(|name| differing.contains(name))
        .cloned()
        .collect();
    if !untracked.is_empty() {
        // An untracked file only matters when the other side tracks a file of that name.
        let incoming =
            names(&["ls-tree", "-r", "-z", "--name-only", &destination]).unwrap_or_default();
        blocking.extend(
            untracked
                .iter()
                .filter(|name| incoming.contains(name))
                .cloned(),
        );
    }
    let words: Vec<&str> = target.iter().map(String::as_str).collect();
    let switch = repo.shell(&[&["checkout"], &words[..]].concat());
    let after = repo.shell(&["status", "--short", "--branch"]);
    let go = Action::Run(format!("{switch} && {after}"));
    if changed.is_empty() && blocking.is_empty() {
        let question = format!("Check out {what}?");
        let choices = vec![choice(&format!("Check out {shown}"), "", go), cancel()];
        return ask(&question, &question, choices, rows);
    }
    let in_the_way: Vec<&String> = blocking
        .iter()
        .filter(|name| untracked.contains(name))
        .collect();
    let message = format!("before checkout of {shown}");
    let mut stash = vec!["stash", "push"];
    if !in_the_way.is_empty() {
        stash.extend(["--include-untracked", "-m", &message, "--"]);
        stash.extend(
            changed
                .iter()
                .chain(in_the_way.iter().copied())
                .map(String::as_str),
        );
    } else {
        stash.extend(["-m", &message]);
    }
    let mut facts = vec![format!(
        "You have {}.",
        plural(changed.len(), "changed file")
    )];
    facts.retain(|_| !changed.is_empty());
    if !in_the_way.is_empty() {
        facts.push(format!(
            "{} would be overwritten.",
            plural(in_the_way.len(), "untracked file")
        ));
    }
    let anyway = match blocking.len() {
        0 => "your changes come along".to_string(),
        count => format!(
            "Git will refuse: {}{} differ{} in {shown}",
            blocking[..count.min(3)].join(", "),
            if count > 3 { " and more" } else { "" },
            if count == 1 { "s" } else { "" }
        ),
    };
    let stashed = Action::Run(format!("{} && {switch} && {after}", repo.shell(&stash)));
    let choices = vec![
        choice("Check out anyway", &anyway, go),
        choice(
            "Stash, then check out",
            &format!("saved as \"{message}\""),
            stashed,
        ),
        cancel(),
    ];
    let question = format!("Check out {what}? {}", facts.join(" "));
    ask(&question, &question, choices, rows)
}

#[cfg(test)]
mod tests;
