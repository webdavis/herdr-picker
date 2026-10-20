use std::env;
use std::path::PathBuf;

use herdr_picker_adapters::{
    bookmarks_file, captures_dir, contacts_file, on_path, places_file, quickfix_file,
};
use herdr_picker_protocol::SOCKET_PATH_VARIABLE;

use crate::agent_hook::snapshot_path;
use crate::command::{CommandError, failed, state_dir};

pub(crate) struct Places {
    pub(crate) cwd: PathBuf,
    pub(crate) home: Option<PathBuf>,
    pub(crate) fd: Option<PathBuf>,
    pub(crate) bat: Option<PathBuf>,
    pub(crate) bookmarks: PathBuf,
    pub(crate) quickfix: PathBuf,
    pub(crate) captures: PathBuf,
    // The apple picker's copy of the address book.
    pub(crate) contacts: PathBuf,
    pub(crate) herdr: HerdrPlaces,
}

// Where the agents picker finds herdr and what the plugin saved about it.
pub(crate) struct HerdrPlaces {
    // None outside herdr.
    pub(crate) socket: Option<PathBuf>,
    pub(crate) tab: Option<String>,
    pub(crate) snapshot: PathBuf,
    pub(crate) places: PathBuf,
    // The herdr CLI, which previews an agent's latest reply.
    pub(crate) bin: String,
}

impl HerdrPlaces {
    #[cfg(test)]
    pub(crate) fn outside(state: &std::path::Path) -> Self {
        Self {
            socket: None,
            tab: None,
            snapshot: state.join("agents.json"),
            places: state.join("places"),
            bin: "herdr".to_string(),
        }
    }
}

impl Places {
    pub(crate) fn here() -> Result<Self, CommandError> {
        let home = env::var("HOME").ok().filter(|home| !home.is_empty());
        let state = state_dir();
        let set = |name| {
            env::var(name)
                .ok()
                .filter(|value: &String| !value.is_empty())
        };
        Ok(Self {
            cwd: env::current_dir().map_err(failed)?,
            fd: on_path("fd", env::var_os("PATH").as_deref()),
            bat: on_path("bat", env::var_os("PATH").as_deref()),
            bookmarks: bookmarks_file(&state),
            quickfix: quickfix_file(&state),
            captures: captures_dir(&state),
            contacts: contacts_file(&state),
            herdr: HerdrPlaces {
                socket: set(SOCKET_PATH_VARIABLE).map(PathBuf::from),
                tab: set("HERDR_TAB_ID"),
                snapshot: snapshot_path(),
                places: places_file(&state),
                bin: set("HERDR_BIN_PATH").unwrap_or_else(|| "herdr".to_string()),
            },
            home: home.map(PathBuf::from),
        })
    }
}
