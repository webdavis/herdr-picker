use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use pwe_domain::test_engine::{Retention, StoredRun, runs_to_drop};

use crate::private::create_private;

const RUN_FILE_EXTENSION: &str = "jsonl";

pub struct RunFile {
    file: File,
}

impl RunFile {
    pub fn create(folder: &Path, run_id: &str) -> io::Result<RunFile> {
        let file = create_private(&run_file(folder, run_id))?;
        Ok(RunFile { file })
    }

    pub fn append(&mut self, line: &str) -> io::Result<()> {
        self.file.write_all(format!("{line}\n").as_bytes())?;
        self.file.flush()
    }
}

pub fn run_folder(state_folder: &Path, repo_root: &Path) -> PathBuf {
    let repo_key = repo_root.to_string_lossy().replace('/', "%");
    state_folder.join("tests").join(repo_key)
}

pub fn drop_old_runs(folder: &Path, retention: &Retention) -> io::Result<()> {
    let stored: Vec<StoredRun> = fs::read_dir(folder)?
        .filter_map(|entry| stored_run(&entry.ok()?.path()))
        .collect();
    for run_id in runs_to_drop(retention, &stored) {
        fs::remove_file(run_file(folder, &run_id))?;
    }
    Ok(())
}

fn run_file(folder: &Path, run_id: &str) -> PathBuf {
    folder.join(format!("{run_id}.{RUN_FILE_EXTENSION}"))
}

fn stored_run(path: &Path) -> Option<StoredRun> {
    if path.extension()? != RUN_FILE_EXTENSION {
        return None;
    }
    Some(StoredRun {
        run_id: path.file_stem()?.to_str()?.to_string(),
        bytes: path.metadata().ok()?.len(),
    })
}

#[cfg(test)]
mod tests;
