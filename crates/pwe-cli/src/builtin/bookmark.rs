use std::path::{Path, PathBuf};

use pwe_adapters::add_bookmark;

use super::Places;
use super::rows::in_home;
use crate::command::{CommandError, failed};

pub(crate) fn bookmark(folder: &Path, places: &Places) -> Result<String, CommandError> {
    let shown = in_home(folder, places.home.as_deref());
    Ok(
        match add_bookmark(&places.bookmarks, folder).map_err(failed)? {
            true => format!("Bookmarked {shown}"),
            false => format!("Already bookmarked {shown}"),
        },
    )
}

pub(crate) fn bookmark_folder(active_pane_cwd: Option<String>, cwd: &Path) -> PathBuf {
    active_pane_cwd
        .filter(|folder| !folder.is_empty())
        .map_or_else(|| cwd.to_path_buf(), PathBuf::from)
}
