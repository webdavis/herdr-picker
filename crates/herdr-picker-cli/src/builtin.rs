use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use herdr_picker_adapters::{add_bookmark, bookmarks_file, on_path};
use herdr_picker_domain::{Action, BUILTIN_PICKERS, PickerRow, Preview};
use herdr_picker_protocol::{parse_action, shell_word};

use crate::command::{CommandError, failed};

mod files;
mod folders;
mod rows;

use rows::{RowWriter, in_home};

pub(crate) struct Places {
    pub(crate) cwd: PathBuf,
    pub(crate) home: Option<PathBuf>,
    pub(crate) fd: Option<PathBuf>,
    pub(crate) bookmarks: PathBuf,
}

impl Places {
    pub(crate) fn here() -> Result<Self, CommandError> {
        let home = env::var("HOME").ok().filter(|home| !home.is_empty());
        let state = env::var("HERDR_PLUGIN_STATE_DIR").ok();
        Ok(Self {
            cwd: env::current_dir().map_err(failed)?,
            fd: on_path("fd", env::var_os("PATH").as_deref()),
            bookmarks: bookmarks_file(state.as_deref(), home.as_deref()),
            home: home.map(PathBuf::from),
        })
    }
}

pub(crate) fn print_rows(
    name: &str,
    arguments: &[&str],
    source: Option<&str>,
    places: &Places,
    out: &mut impl Write,
) -> Result<(), CommandError> {
    let first = BUILTIN_PICKERS
        .iter()
        .find(|builtin| builtin.name == name)
        .and_then(|builtin| builtin.sources.first().copied());
    let mut rows = RowWriter::new(out);
    match (name, arguments, source.or(first)) {
        ("files", [], Some(source)) => files::files(source, places, &mut rows)?,
        ("folders", [], Some(source)) => folders::folders(source, places, &mut rows)?,
        ("echo-args", words, _) => written(
            words
                .iter()
                .try_for_each(|word| rows.row(&row(word, None, Action::Insert(shell_word(word))))),
        )?,
        ("confirm", [question, action], _) => {
            let action = parse_action(action).map_err(failed)?;
            written(rows.row(&row("Yes", Some(question), action)))?;
            written(rows.row(&row("No", Some(question), Action::Back)))?;
        }
        _ => return Err(failed(format!("no builtin picker is named {name}"))),
    }
    written(rows.finish())
}

// The finder closing ends the list early; that is no failure.
fn written(result: io::Result<()>) -> Result<(), CommandError> {
    match result {
        Err(failure) if failure.kind() != io::ErrorKind::BrokenPipe => Err(failed(failure)),
        _ => Ok(()),
    }
}

fn row(label: &str, preview: Option<&str>, enter: Action) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: preview.map(|text| Preview::Text(text.to_string())),
        enter,
        keys: Vec::new(),
        group: None,
    }
}

pub(crate) fn bookmark(folder: &Path, places: &Places) -> Result<String, CommandError> {
    let shown = in_home(folder, places.home.as_deref());
    Ok(
        match add_bookmark(&places.bookmarks, folder).map_err(failed)? {
            true => format!("Bookmarked {shown}"),
            false => format!("Already bookmarked {shown}"),
        },
    )
}

// herdr names the focused pane's folder to a plugin action; a shell key binding runs in the shell's.
pub(crate) fn bookmark_folder(active_pane_cwd: Option<String>, cwd: &Path) -> PathBuf {
    active_pane_cwd
        .filter(|folder| !folder.is_empty())
        .map_or_else(|| cwd.to_path_buf(), PathBuf::from)
}

#[cfg(test)]
mod tests;
