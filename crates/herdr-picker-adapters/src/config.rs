use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use herdr_picker_domain::{JumpTarget, Picker, RowLook, Theme, WorkspaceDeclaration, jump_targets};
use serde::Deserialize;

mod error;
mod pickers;

pub use error::ConfigError;
use pickers::{PickerEntry, declared_pickers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    pub targets: Vec<JumpTarget>,
    pub look: RowLook,
    pub theme: Theme,
    pub pickers: Vec<Picker>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    workspaces: Option<BTreeMap<String, Declaration>>,
    #[serde(default)]
    picker: PickerTable,
    #[serde(default)]
    pickers: BTreeMap<String, PickerEntry>,
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
#[derive(Deserialize)]
#[serde(
    untagged,
    expecting = "a directory, or a table of a dir and an optional key"
)]
enum Declaration {
    Directory(String),
    Detailed(DetailedDeclaration),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DetailedDeclaration {
    dir: String,
    key: Option<String>,
}

impl Declaration {
    fn labelled(self, label: String) -> WorkspaceDeclaration {
        match self {
            Self::Directory(directory) => WorkspaceDeclaration {
                label,
                directory,
                key: None,
            },
            Self::Detailed(detailed) => WorkspaceDeclaration {
                label,
                directory: detailed.dir,
                key: detailed.key,
            },
        }
    }
}
pub fn config_file(
    plugin_config_dir: Option<&str>,
    xdg_config_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    let directory = match plugin_config_dir.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => base_config_dir(xdg_config_home, home).join("herdr/plugins/config/herdr-picker"),
    };
    directory.join("config.toml")
}

fn base_config_dir(xdg_config_home: Option<&str>, home: Option<&str>) -> PathBuf {
    match xdg_config_home.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => PathBuf::from(home.unwrap_or_default()).join(".config"),
    }
}
pub fn read_config(path: &Path) -> Result<PluginConfig, ConfigError> {
    let content = fs::read_to_string(path).map_err(|failure| ConfigError::Unreadable {
        path: path.to_owned(),
        reason: failure.to_string(),
    })?;
    let file: ConfigFile =
        toml::from_str(&content).map_err(|failure| ConfigError::Unparseable {
            path: path.to_owned(),
            reason: failure.message().to_string(),
            line: one_based_line_of(&content, &failure),
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
    let pickers =
        declared_pickers(file.pickers, &path.with_file_name("pickers")).map_err(|cause| {
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
    })
}
fn one_based_line_of(content: &str, failure: &toml::de::Error) -> Option<usize> {
    let span = failure.span()?;
    let before = content.get(..span.start)?;
    Some(before.matches('\n').count() + 1)
}

#[cfg(test)]
mod tests;
