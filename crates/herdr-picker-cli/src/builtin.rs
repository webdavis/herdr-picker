use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use herdr_picker_adapters::add_bookmark;
use herdr_picker_domain::{Action, BUILTIN_PICKERS, PickerRow, Preview};
use herdr_picker_protocol::{
    PICKER_QUERY_VARIABLE, PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE, parse_action, shell_word,
};

use crate::command::{CommandError, failed};

pub(crate) mod agents;
mod clock;
mod data;
mod files;
mod folders;
mod git;
mod network;
mod page;
mod paint;
mod places;
mod processes;
mod program;
mod project;
mod recipes;
mod rows;

pub(crate) use data::data_command;
pub(crate) use git::git_preview;
pub(crate) use network::network_tool;
pub(crate) use places::{HerdrPlaces, Places};
pub(crate) use project::save_check;

use rows::{RowWriter, in_home};

// What the step asked of its picker: the source to read, its state, and what is typed.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Asked<'a> {
    pub(crate) source: Option<&'a str>,
    pub(crate) state: Option<&'a str>,
    pub(crate) query: &'a str,
}

// What the step asked is in the variables herdr-picker set for the builtin's run.
pub(crate) fn print_asked(name: &str, arguments: &[&str]) -> Result<(), CommandError> {
    let read = |variable| env::var(variable).ok();
    let (source, state) = (read(PICKER_SOURCE_VARIABLE), read(PICKER_STATE_VARIABLE));
    let query = read(PICKER_QUERY_VARIABLE).unwrap_or_default();
    let asked = Asked {
        source: source.as_deref(),
        state: state.as_deref(),
        query: &query,
    };
    print_rows(
        name,
        arguments,
        &asked,
        &Places::here()?,
        &mut io::stdout().lock(),
    )
}

pub(crate) fn print_rows(
    name: &str,
    arguments: &[&str],
    asked: &Asked,
    places: &Places,
    out: &mut impl Write,
) -> Result<(), CommandError> {
    let first = BUILTIN_PICKERS
        .iter()
        .find(|builtin| builtin.name == name)
        .and_then(|builtin| builtin.sources.first().copied());
    let mut rows = RowWriter::new(out);
    let asked = Asked {
        source: asked.source.or(first),
        ..*asked
    };
    let step = git::git_rows(name, arguments, &asked, places, &mut rows)
        .or_else(|| network::network_step(name, arguments, &asked, places, &mut rows))
        .or_else(|| data::data_rows(name, arguments, &asked, places, &mut rows));
    if let Some(listed) = step {
        listed?;
        return written(rows.finish());
    }
    match (name, arguments, asked.source) {
        (name @ ("agents" | "send-to" | "quick-answer" | "answer"), [], _) => {
            agents::agents(name, &asked, places, &mut rows)?
        }
        ("files", [], Some(source)) => files::files(source, places, &mut rows)?,
        ("folders", [], Some(source)) => folders::folders(source, places, &mut rows)?,
        ("recipes", [], _) => recipes::recipes_rows(places, &mut rows)?,
        ("tests", [], Some(source)) => project::tests(source, places, &mut rows)?,
        ("processes", [], Some(source)) => processes::processes(source, &mut rows)?,
        ("network", [], Some(source)) => network::network(source, &asked, places, &mut rows)?,
        ("unified-logs", [], _) => processes::unified_logs(&mut rows)?,
        ("crashes", [], _) => processes::crashes(places, &mut rows)?,
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
