use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process;

// What a picker saves can name the user's files and say what their checks printed: only they read it.
// It lands whole, so a reader never sees half of it.
pub(crate) fn write_private(file: &Path, contents: &[u8]) -> io::Result<()> {
    let mut temporary = file.as_os_str().to_owned();
    temporary.push(format!(".{}.tmp", process::id()));
    let temporary = PathBuf::from(temporary);
    create_private(&temporary)?.write_all(contents)?;
    fs::rename(&temporary, file)
}

pub(crate) fn create_private(file: &Path) -> io::Result<File> {
    if let Some(directory) = file.parent() {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)?;
    }
    let created = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(file)?;
    created.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(created)
}
