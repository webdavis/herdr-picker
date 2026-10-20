use std::io::Write;

use herdr_picker_domain::{Action, PickerRow, Preview};
use herdr_picker_protocol::shell_word;

use super::paint::dim;
use super::steps::confirm;
use super::{Repo, plural, strings};
use crate::builtin::rows::RowWriter;
use crate::builtin::{Places, written};
use crate::command::CommandError;

// The steps where what is typed is the answer: a stash message, a branch's new name, a
// setting's new value. Each shows one row for it, which asks before anything changes.
pub(super) fn stash_files(
    names: &[&str],
    message: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    let mut new = vec!["ls-files", "-z", "--others", "--exclude-standard", "--"];
    new.extend(names);
    let untracked = !strings(&repo.git(&new)?).is_empty();
    let mut stash = vec!["stash", "push"];
    stash.extend(untracked.then_some("--include-untracked"));
    stash.extend(match message.trim() {
        "" => vec![],
        message => vec!["-m", message],
    });
    stash.push("--");
    stash.extend(names);
    let files = plural(names.len(), "file");
    let what = match message.trim() {
        "" => format!("Stash {files}"),
        message => format!("Stash {files} as \"{message}\""),
    };
    let shown = names[..names.len().min(3)].join(", ");
    let more = if names.len() > 3 { " and more" } else { "" };
    answer(
        "Type a message for the stash, or leave it empty, then press Enter.",
        PickerRow {
            preview: Some(Preview::Text(format!("{shown}{more}"))),
            ..asking(&what, &format!("{what}?"), repo.shell(&stash))
        },
        rows,
    )
}

pub(super) fn rename_branch(
    old: &str,
    new: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    let new = new.trim();
    let row = match new.is_empty() || new == old {
        true => PickerRow {
            label: dim("Type a new name for the branch."),
            preview: None,
            enter: Action::Nothing,
            keys: Vec::new(),
            group: None,
        },
        false => asking(
            &format!("Rename {old} to {new}"),
            &format!("Rename {old} to {new}?"),
            repo.shell(&["branch", "-m", old, new]),
        ),
    };
    answer(
        "Type the branch's new name, then press Enter to review the change.",
        row,
        rows,
    )
}

pub(super) fn change_setting(
    [file, name, scope, shown]: [&str; 4],
    new: &str,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let command = ["git", "config", "--file", file, name, new]
        .map(shell_word)
        .join(" ");
    let row = asking(
        &format!("Set {name} to {new}"),
        &format!("Set {name} in {scope} from {shown} to {new}?"),
        command,
    );
    answer(
        "Type the new value, then press Enter to review the change.",
        row,
        rows,
    )
}

fn asking(label: &str, question: &str, command: String) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: Some(Preview::Text(question.to_string())),
        enter: confirm(question, command),
        keys: Vec::new(),
        group: None,
    }
}

fn answer(
    prompt: &str,
    row: PickerRow,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    written(rows.header(prompt).and_then(|()| rows.row(&row)))
}

#[cfg(test)]
mod tests;
