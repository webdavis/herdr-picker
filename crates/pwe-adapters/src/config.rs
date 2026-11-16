use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use pwe_domain::{InboxLimits, JumpTarget, Picker, RowLook, Theme, jump_targets};
use serde::Deserialize;

mod error;
mod location;
mod notices;
mod pickers;
mod rex;
mod services;
mod workspaces;

pub use error::ConfigError;
pub use location::config_file;
pub use notices::NoticeConfig;
use notices::{InboxTable, NoticesTable};
use pickers::{PickerEntry, declared_pickers};
pub use rex::RexSettings;
use services::service_tables;
pub use services::{BlueBubblesSettings, UnifiSettings};
use workspaces::Declaration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    pub targets: Vec<JumpTarget>,
    pub look: RowLook,
    pub theme: Theme,
    pub pickers: Vec<Picker>,
    pub unifi: Option<UnifiSettings>,
    pub bluebubbles: Option<BlueBubblesSettings>,
    pub rex: RexSettings,
    pub notices: NoticeConfig,
    pub inbox: InboxLimits,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    workspaces: Option<BTreeMap<String, Declaration>>,
    #[serde(default)]
    picker: PickerTable,
    #[serde(default)]
    pickers: BTreeMap<String, PickerEntry>,
    #[serde(default)]
    rex: RexSettings,
    #[serde(default)]
    notices: NoticesTable,
    #[serde(default)]
    inbox: InboxTable,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct PickerTable {
    #[serde(default)]
    leaders: bool,
    #[serde(default)]
    stripes: bool,
    #[serde(default)]
    theme: ThemeName,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum ThemeName {
    #[default]
    CatppuccinMocha,
}

impl From<ThemeName> for Theme {
    fn from(name: ThemeName) -> Self {
        match name {
            ThemeName::CatppuccinMocha => Self::CatppuccinMocha,
        }
    }
}
pub fn read_config(path: &Path) -> Result<PluginConfig, ConfigError> {
    let content = fs::read_to_string(path).map_err(|failure| ConfigError::Unreadable {
        path: path.to_owned(),
        reason: failure.to_string(),
    })?;
    parsed(path, &content)
}

pub fn read_picker_config(path: &Path) -> Result<PluginConfig, ConfigError> {
    match fs::exists(path) {
        Ok(false) => parsed(path, ""),
        _ => read_config(path),
    }
}

fn parsed(path: &Path, content: &str) -> Result<PluginConfig, ConfigError> {
    let file: ConfigFile = toml::from_str(content).map_err(|failure| ConfigError::Unparseable {
        path: path.to_owned(),
        reason: failure.message().to_string(),
        line: one_based_line_of(content, &failure),
    })?;
    let targets = match file.workspaces {
        None => Vec::new(),
        Some(workspaces) => jump_targets(
            workspaces
                .into_iter()
                .map(|(label, declaration)| declaration.labelled(label)),
        )
        .map_err(|cause| ConfigError::Invalid {
            path: path.to_owned(),
            cause,
        })?,
    };
    let mut entries = file.pickers;
    let (unifi, bluebubbles) = service_tables(&mut entries, path)?;
    let pickers = declared_pickers(entries, &path.with_file_name("pickers")).map_err(|cause| {
        ConfigError::InvalidPicker {
            path: path.to_owned(),
            cause,
        }
    })?;
    let PickerTable {
        leaders,
        stripes,
        theme,
    } = file.picker;
    Ok(PluginConfig {
        targets,
        look: RowLook { leaders, stripes },
        theme: theme.into(),
        pickers,
        unifi,
        bluebubbles,
        rex: file.rex,
        notices: file.notices.settings(path)?,
        inbox: file.inbox.limits(),
    })
}
fn one_based_line_of(content: &str, failure: &toml::de::Error) -> Option<usize> {
    let span = failure.span()?;
    let before = content.get(..span.start)?;
    Some(before.matches('\n').count() + 1)
}

#[cfg(test)]
mod tests;
