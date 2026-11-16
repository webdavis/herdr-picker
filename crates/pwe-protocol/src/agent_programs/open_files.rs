use std::path::PathBuf;

pub fn open_files_in_lsof_listing(printed: &str) -> Vec<(u32, PathBuf)> {
    let mut pid = None;
    let mut files = Vec::new();
    for line in printed.lines() {
        if let Some(number) = line.strip_prefix('p') {
            pid = number.parse().ok();
        } else if let (Some(name), Some(pid)) = (line.strip_prefix('n'), pid) {
            files.push((pid, PathBuf::from(name)));
        }
    }
    files
}

#[cfg(test)]
mod tests;
