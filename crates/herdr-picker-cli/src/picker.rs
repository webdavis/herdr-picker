use std::path::Path;
use std::{env, fs, process};

use herdr_picker_adapters::{
    Chrome, Placement, copy_to_clipboard, fzf_arguments, picker_rows_file, read_config,
    remove_run_files, run_picker,
};
use herdr_picker_domain::Action;
use herdr_picker_protocol::{bash_init, instruction, parse_row};

use crate::command::{CommandError, config_path, failed};
use crate::picker_chrome::QuotedHelper;
use crate::picker_popup;

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

pub(crate) fn pick(
    name: &str,
    config: &Path,
    placement: Placement,
) -> Result<Option<String>, CommandError> {
    let config = read_config(config).map_err(failed)?;
    let picker = config
        .pickers
        .iter()
        .find(|picker| picker.name == name)
        .ok_or_else(|| CommandError::Failed(format!("no picker is named {name}")))?;
    let cache = picker_rows_file(
        env::var("HERDR_PLUGIN_STATE_DIR").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
        process::id(),
    );
    let quoted = QuotedHelper::for_run(&cache)?;
    let chrome = Chrome {
        helper: quoted.helper(),
        name,
        description: &picker.description,
    };
    let finder_arguments = fzf_arguments(config.theme, &chrome, placement);
    remove_run_files(&cache);
    let picked = run_picker(&picker.command, Path::new("fzf"), &finder_arguments, &cache);
    remove_run_files(&cache);
    let picked = picked.map_err(|failure| failed(format!("picker {name}: {failure}")))?;
    match picked.map(|row| row.enter) {
        Some(Action::Copy(text)) => copy_to_clipboard(&text).map(|()| None).map_err(failed),
        action => Ok(action.as_ref().and_then(instruction)),
    }
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
