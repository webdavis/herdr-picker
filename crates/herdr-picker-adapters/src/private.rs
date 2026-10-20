use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

// What a picker saves can name the user's files and say what their checks printed: only they read it.
pub(crate) fn write_private(file: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(directory) = file.parent() {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)?;
    }
    let mut saved = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(file)?;
    saved.set_permissions(fs::Permissions::from_mode(0o600))?;
    saved.write_all(contents)
}
