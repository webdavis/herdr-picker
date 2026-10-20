use std::path::Path;
use std::{env, fs, process};

use herdr_picker_adapters::{Placement, copy_to_clipboard, picker_rows_file, read_config};
use herdr_picker_domain::{Action, PickerCall, Trail, Turn};
use herdr_picker_protocol::{Shell, bash_init, instruction, parse_row};

use crate::command::{CommandError, config_path, failed};
use crate::picker_chrome::QuotedHelper;
use crate::{picker_popup, picker_step};

const OPENER: &str = if cfg!(target_os = "macos") {
    "open"
} else {
    "xdg-open"
};

pub(crate) fn run(socket_path: Option<String>, name: &str) -> Result<(), CommandError> {
    let config = config_path();
    let instruction = match picker_popup::popup_socket(socket_path) {
        Some(socket) => picker_popup::pick(&socket, name, &config)?,
        None => pick(name, &config, Placement::Inline)?,
    };
    if let Some(instruction) = instruction {
        println!("{instruction}");
    }
    Ok(())
}

// Each step is one finder run; next, confirm, reload and Esc start the next run.
pub(crate) fn pick(
    name: &str,
    config: &Path,
    placement: Placement,
) -> Result<Option<String>, CommandError> {
    let config = read_config(config).map_err(failed)?;
    let cache = picker_rows_file(
        env::var("HERDR_PLUGIN_STATE_DIR").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
        process::id(),
    );
    let quoted = QuotedHelper::for_run(&cache)?;
    let mut trail = Trail::new(PickerCall {
        name: name.to_string(),
        args: Vec::new(),
        state: None,
    });
    let step = picker_step::Run {
        config: &config,
        helper: quoted.helper(),
        cache: &cache,
        placement,
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
    let editor = ["EDITOR_CMD", "EDITOR"]
        .iter()
        .find_map(|name| {
            env::var(name)
                .ok()
                .filter(|editor| !editor.trim().is_empty())
        })
        .unwrap_or_else(|| "nvim".to_string());
    let cwd = env::current_dir().map_err(failed)?;
    let shell = Shell {
        editor: &editor,
        cwd: &cwd.to_string_lossy(),
        opener: OPENER,
    };
    instruction(action, &shell)
        .map_err(|refusal| failed(format!("the editor {editor:?}: {refusal}")))
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
    Ok(preview.unwrap_or_default())
}

pub(crate) fn init(shell: &str, config: &Path) -> Result<String, CommandError> {
    if shell != "bash" {
        return Err(CommandError::Usage);
    }
    Ok(bash_init(&read_config(config).map_err(failed)?.pickers))
}

#[cfg(test)]
mod tests;
