use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use pwe_domain::RecordedPane;
use pwe_protocol::{agent_panes_json, parse_agent_panes};

use crate::private::write_private_atomically;

const ONLY_ITS_OWNER: u32 = 0o600;

pub fn save_agent_panes(file: &Path, panes: &[RecordedPane]) -> io::Result<()> {
    write_private_atomically(file, agent_panes_json(panes).as_bytes())
}

pub fn read_agent_panes(file: &Path) -> io::Result<Vec<RecordedPane>> {
    let found = match fs::symlink_metadata(file) {
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        found => found?,
    };
    let private = found.is_file()
        && found.permissions().mode() & 0o777 == ONLY_ITS_OWNER
        && found.uid() == fs::metadata(file.parent().unwrap_or(Path::new("/")))?.uid();
    if !private {
        return Err(io::Error::other(format!(
            "{} is not a file only its owner may read and write",
            file.display()
        )));
    }
    Ok(parse_agent_panes(&fs::read_to_string(file)?))
}

#[cfg(test)]
mod tests;
