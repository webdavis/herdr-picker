use std::env;
use std::path::PathBuf;

use herdr_picker_adapters::{bookmarks_file, captures_dir, on_path, quickfix_file};

use crate::command::{CommandError, failed};

pub(crate) struct Places {
    pub(crate) cwd: PathBuf,
    pub(crate) home: Option<PathBuf>,
    pub(crate) fd: Option<PathBuf>,
    pub(crate) bat: Option<PathBuf>,
    pub(crate) bookmarks: PathBuf,
    pub(crate) quickfix: PathBuf,
    pub(crate) captures: PathBuf,
}

impl Places {
    pub(crate) fn here() -> Result<Self, CommandError> {
        let home = env::var("HOME").ok().filter(|home| !home.is_empty());
        let state = env::var("HERDR_PLUGIN_STATE_DIR").ok();
        Ok(Self {
            cwd: env::current_dir().map_err(failed)?,
            fd: on_path("fd", env::var_os("PATH").as_deref()),
            bat: on_path("bat", env::var_os("PATH").as_deref()),
            bookmarks: bookmarks_file(state.as_deref(), home.as_deref()),
            quickfix: quickfix_file(state.as_deref(), home.as_deref()),
            captures: captures_dir(state.as_deref(), home.as_deref()),
            home: home.map(PathBuf::from),
        })
    }
}
