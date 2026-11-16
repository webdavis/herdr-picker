use std::path::Path;
use std::{env, fs, process};

use pwe_adapters::{
    FzfStepFinder, Placement, bare_program, command_preview, copy_to_clipboard, path_preview,
    picker_rows_file, read_picker_config, remove_runs_whose_process_is_gone,
};
use pwe_domain::{Action, Picker, PickerCall, PickerCommand, Preview, Trail, Turn};
use pwe_protocol::{Shell, bash_init, instruction, parse_row};

use crate::builtin::{self, Places};
use crate::command::{CommandError, config_path, failed, state_dir};
use crate::picker_chrome::QuotedHelper;
use crate::quickfix::quickfix_path;
use crate::{handing, picker_popup, picker_step};

const OPENER: &str = if cfg!(target_os = "macos") {
    "open"
} else {
    "xdg-open"
};

pub(crate) fn run(socket_path: Option<String>, name: &str) -> Result<(), CommandError> {
    let config = config_path();
    if is_bookmark(name, &read_picker_config(&config).map_err(failed)?.pickers) {
        eprintln!("{}", bookmark_this_folder()?);
        return Ok(());
    }
    let socket = picker_popup::popup_socket(socket_path);
    let instruction = match &socket {
        Some(socket) => picker_popup::pick(socket, name, &config)?,
        None => pick(name, &config, Placement::Inline)?,
    };
    handing::answer(socket.as_deref(), instruction)
}

pub(crate) fn is_bookmark(name: &str, pickers: &[Picker]) -> bool {
    let bookmarks = PickerCommand::Builtin("bookmark".to_string());
    pickers
        .iter()
        .any(|picker| picker.name == name && picker.command == bookmarks)
}

pub(crate) fn bookmark_this_folder() -> Result<String, CommandError> {
    let places = Places::here()?;
    let folder = builtin::bookmark_folder(env::var("HERDR_ACTIVE_PANE_CWD").ok(), &places.cwd);
    builtin::bookmark(&folder, &places)
}

pub(crate) fn pick(
    name: &str,
    config: &Path,
    placement: Placement,
) -> Result<Option<String>, CommandError> {
    let config = read_picker_config(config).map_err(failed)?;
    remove_runs_whose_process_is_gone(&state_dir());
    let cache = picker_rows_file(&state_dir(), process::id());
    let quoted = QuotedHelper::for_run(&cache)?;
    let mut trail = Trail::new(PickerCall {
        name: name.to_string(),
        args: Vec::new(),
        state: None,
        query: None,
    });
    let quickfix = quickfix_path();
    let finder = FzfStepFinder {
        helper: quoted.helper(),
        cache: &cache,
        theme: config.theme,
        placement,
    };
    let step = picker_step::Run {
        config: &config,
        finder: &finder,
        quickfix: &quickfix,
    };
    loop {
        let Some((action, spot)) = step.show(&trail)? else {
            return Ok(None);
        };
        match trail.advance(action, spot) {
            Turn::Show => {}
            Turn::Close => return Ok(None),
            Turn::Leave(action) => return finish(&action),
        }
    }
}

fn finish(action: &Action) -> Result<Option<String>, CommandError> {
    if let Action::Copy(text) = action {
        return copy_to_clipboard(text).map(|()| None).map_err(failed);
    }
    let editor = editor();
    let cwd = env::current_dir().map_err(failed)?;
    let shell = Shell {
        editor: &editor,
        cwd: &cwd.to_string_lossy(),
        opener: OPENER,
    };
    instruction(action, &shell)
        .map_err(|refusal| failed(format!("the editor {editor:?}: {refusal}")))
}

pub(crate) fn editor() -> String {
    let editor = ["EDITOR_CMD", "EDITOR"]
        .iter()
        .find_map(|name| {
            env::var(name)
                .ok()
                .filter(|editor| !editor.trim().is_empty())
        })
        .unwrap_or_else(|| "nvim".to_string());
    bare_program(&editor, env::var_os("PATH").as_deref())
}

pub(crate) fn preview(cache: &Path, field: &str) -> Result<String, CommandError> {
    let rows = fs::read_to_string(cache).map_err(failed)?;
    let index = field
        .trim()
        .parse::<usize>()
        .map_err(|_| CommandError::Usage)?;
    let preview = rows
        .lines()
        .nth(index)
        .and_then(|line| parse_row(line, index + 1).ok())
        .and_then(|row| row.preview);
    Ok(match preview {
        None => String::new(),
        Some(Preview::Text(text)) => text,
        Some(Preview::Path(path)) => {
            let width = env::var("FZF_PREVIEW_COLUMNS").ok();
            let width = width.and_then(|columns| columns.parse().ok());
            path_preview(Path::new(&path), env::var_os("PATH").as_deref(), width)
        }
        Some(Preview::Command(argv)) => command_preview(&argv),
    })
}

pub(crate) fn init(shell: &str, config: &Path) -> Result<String, CommandError> {
    if shell != "bash" {
        return Err(CommandError::Usage);
    }
    Ok(bash_init(
        &read_picker_config(config).map_err(failed)?.pickers,
    ))
}

#[cfg(test)]
mod tests;
