use std::env;
use std::path::Path;

use pwe_adapters::{
    Layout, PaneHost, Placement, RexPanes, TerminalInput, TerminalScreen, hand_over,
    read_picker_config, rex_program,
};
use pwe_application::{InputSource, pick};
use pwe_domain::{Fuzzy, LauncherKeymap, Menu, ReadlineField, launcher_base, launcher_targets};

use crate::agent_hook::queue_path;
use crate::command::{CommandError, config_path, failed};
use crate::picker;

const APPLICATIONS: &str = "/Applications";

pub(crate) fn launcher(base: &str, block: &str) -> Result<(), CommandError> {
    let outcome = pick_and_hand_over(base, block);
    if let Err(CommandError::Failed(reason)) = &outcome {
        show_until_a_key(reason);
    }
    outcome
}

fn pick_and_hand_over(base: &str, block: &str) -> Result<(), CommandError> {
    let stroke = launcher_base(base).ok_or(CommandError::Usage)?;
    let config_file = config_path();
    let config = read_picker_config(&config_file).map_err(failed)?;
    let offered = launcher_targets(&config.pickers, stroke);
    let layout = Layout::new(LauncherKeymap::LEGEND, &offered)
        .look(config.look)
        .theme(config.theme);
    let menu = Menu::new(&offered, &[], None, Fuzzy, ReadlineField::default());
    let mut screen = TerminalScreen::stdout(layout);
    let Some(chosen) =
        pick(menu, &LauncherKeymap, &mut TerminalInput, &mut screen).map_err(failed)?
    else {
        return Ok(());
    };
    if picker::is_bookmark(&chosen.label, &config.pickers) {
        show_until_a_key(&picker::bookmark_this_folder()?);
        return Ok(());
    }
    let instruction = picker::pick(&chosen.label, &config_file, Placement::Popup)?;
    let path_variable = env::var_os("PATH");
    let rex = rex_program(
        path_variable.as_deref(),
        Path::new(APPLICATIONS),
        &config.rex.app_name,
    )
    .map_err(failed)?;
    let session = env::var("REX_SESSION").ok().filter(|id| !id.is_empty());
    hand_to_block(&mut RexPanes::at(rex, session), block, instruction)
}

pub(crate) fn hand_to_block(
    panes: &mut impl PaneHost,
    block: &str,
    instruction: Option<String>,
) -> Result<(), CommandError> {
    let Some(line) = instruction else {
        return Ok(());
    };
    hand_over(panes, block, &line, &picker::editor(), &queue_path()).map_err(failed)
}

fn show_until_a_key(message: &str) {
    eprintln!("{message}\n\nPress any key to close.");
    let _ = TerminalInput.next_input();
}

#[cfg(test)]
mod tests;
