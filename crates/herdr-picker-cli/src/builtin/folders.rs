use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use herdr_picker_adapters::{Entry, Walk, read_bookmarks, walk};

use super::rows::{RowWriter, folder_row};
use super::{Places, written};
use crate::command::{CommandError, failed};

pub(super) fn folders(
    source: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let mut show = |path: &Path| rows.row(&folder_row(path, places));
    match source {
        "subfolders" => {
            let search = Walk {
                root: places.cwd.clone(),
                entry: Entry::Folders,
                hidden: false,
                depth: None,
            };
            written(walk(&search, places.fd.as_deref(), show))
        }
        "parents" => written(places.cwd.ancestors().skip(1).try_for_each(show)),
        "git" => written(
            tracked_folders(&places.cwd)
                .iter()
                .try_for_each(|path| show(path)),
        ),
        "bookmarks" => written(
            read_bookmarks(&places.bookmarks)
                .iter()
                .filter(|folder| folder.is_dir())
                .try_for_each(|path| show(path)),
        ),
        other => Err(failed(format!(
            "the folders picker has no source named {other}"
        ))),
    }
}

fn tracked_folders(cwd: &Path) -> BTreeSet<PathBuf> {
    let git = |folder: &Path, arguments: &[&str]| {
        let ran = Command::new("git")
            .arg("-C")
            .arg(folder)
            .args(arguments)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        ran.status.success().then_some(ran.stdout)
    };
    let Some(top) = git(cwd, &["rev-parse", "--show-toplevel"]) else {
        return BTreeSet::new();
    };
    let root = PathBuf::from(OsStr::from_bytes(top.trim_ascii_end()));
    let names = git(&root, &["ls-files", "-z", "--cached"]).unwrap_or_default();
    let mut folders = BTreeSet::from([root.clone()]);
    for name in names
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let file = root.join(OsStr::from_bytes(name));
        let above = file
            .ancestors()
            .skip(1)
            .take_while(|folder| folder.starts_with(&root));
        folders.extend(above.map(Path::to_path_buf));
    }
    folders
}

#[cfg(test)]
mod tests;
