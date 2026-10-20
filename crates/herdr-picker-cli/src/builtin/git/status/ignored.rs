use std::io::Write;
use std::process::Stdio;

use herdr_picker_domain::{PickerRow, Preview};

use super::super::paint::dim;
use super::super::{Repo, strings};
use super::edit_at;
use crate::builtin::rows::{RowWriter, file_row, key, painted, shown};
use crate::builtin::{Places, written};
use crate::command::{CommandError, failed};

// Ignored files, a whole ignored folder as one row, each with the rule that ignores it.
pub(super) fn ignored(
    repo: &Repo,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let names = repo.git(&[
        "ls-files",
        "--others",
        "--ignored",
        "--exclude-standard",
        "--directory",
        "-z",
    ])?;
    let mut check = repo.command(&["check-ignore", "-z", "-v", "--stdin"]);
    let mut running = check
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(failed)?;
    if let Some(mut stdin) = running.stdin.take() {
        stdin.write_all(&names).map_err(failed)?;
    }
    let ran = running.wait_with_output().map_err(failed)?;
    if !matches!(ran.status.code(), Some(0 | 1)) {
        return Err(failed(format!(
            "git check-ignore stopped with {}",
            ran.status
        )));
    }
    let fields = strings(&ran.stdout);
    let listed = fields
        .as_chunks::<4>()
        .0
        .iter()
        .map(|[source, line, rule, name]| {
            let path = repo.root.join(name.trim_end_matches('/'));
            let label = format!("{}  {}", painted(&shown(&path, &places.cwd)), dim(rule));
            let mut row: PickerRow = file_row(&path, label, places, None);
            row.preview = Some(Preview::Text(format!(
                "Rule: {rule}\nSource: {source}:{line}"
            )));
            let rule_file = repo.root.join(source);
            row.keys
                .retain(|(claimed, _)| *claimed != key("ctrl-enter"));
            row.keys.insert(
                0,
                (key("ctrl-enter"), edit_at(&rule_file, line.parse().ok())),
            );
            row
        });
    let listed: Vec<PickerRow> = listed.collect();
    written(listed.iter().try_for_each(|row| rows.row(row)))
}
