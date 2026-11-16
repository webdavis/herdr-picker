use pwe_adapters::{read_picker_config, show_notice_pane};
use pwe_domain::{Level, Theme};

use crate::agent_hook;
use crate::command::{CommandError, config_path, failed};

pub(crate) fn run(level: &str, until: &str) -> Result<(), CommandError> {
    let (level, until) = notice_options(level, until)?;
    let theme =
        read_picker_config(&config_path()).map_or_else(|_| Theme::default(), |config| config.theme);
    show_notice_pane(level, theme, until, agent_hook::now).map_err(failed)
}

pub(crate) fn notice_options(level: &str, until: &str) -> Result<(Level, u64), CommandError> {
    let level = Level::parse(level)
        .filter(|level| *level != Level::Quiet)
        .ok_or(CommandError::Usage)?;
    let until = until.parse().map_err(|_| CommandError::Usage)?;
    Ok((level, until))
}

#[cfg(test)]
mod tests;
