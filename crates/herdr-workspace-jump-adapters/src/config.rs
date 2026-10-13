use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use herdr_workspace_jump_domain::{
    JumpTarget, RowLook, TargetError, Theme, WorkspaceDeclaration, jump_targets,
};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    pub targets: Vec<JumpTarget>,
    pub look: RowLook,
    pub theme: Theme,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    workspaces: BTreeMap<String, Declaration>,
    #[serde(default)]
    picker: PickerTable,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Unreadable {
        path: PathBuf,
        reason: String,
    },
    Unparseable {
        path: PathBuf,
        reason: String,
        line: Option<usize>,
    },
    Invalid {
        path: PathBuf,
        cause: TargetError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (path, reason) = match self {
            Self::Unreadable { path, reason } => (path, reason.clone()),
            Self::Unparseable { path, reason, line } => (
                path,
                match line {
                    Some(line) => format!("{reason} at line {line}"),
                    None => reason.clone(),
                },
            ),
            Self::Invalid { path, cause } => (path, cause.to_string()),
        };
        write!(
            formatter,
            "cannot read the workspace list from {}: {reason}",
            path.display()
        )
    }
}
pub fn config_file(
    plugin_config_dir: Option<&str>,
    xdg_config_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    let directory = match plugin_config_dir.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => {
            base_config_dir(xdg_config_home, home).join("herdr/plugins/config/herdr-workspace-jump")
        }
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
    let workspaces = file
        .workspaces
        .into_iter()
        .map(|(label, declaration)| declaration.labelled(label));
    let targets = jump_targets(workspaces).map_err(|cause| ConfigError::Invalid {
        path: path.to_owned(),
        cause,
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
    })
}
fn one_based_line_of(content: &str, failure: &toml::de::Error) -> Option<usize> {
    let span = failure.span()?;
    let before = content.get(..span.start)?;
    Some(before.matches('\n').count() + 1)
}

#[cfg(test)]
mod tests;
