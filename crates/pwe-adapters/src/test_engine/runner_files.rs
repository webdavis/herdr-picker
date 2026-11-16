use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use pwe_domain::test_engine::RunnerSpec;
use pwe_protocol::test_engine::{RunnerFileError, read_runner_file};

const SHIPPED: [(&str, &str); 1] = [("cargo.toml", include_str!("../../runners/cargo.toml"))];
const RUNNER_FILE_EXTENSION: &str = "toml";

#[derive(Debug, Default)]
pub struct RunnerFiles {
    pub runners: Vec<RunnerSpec>,
    pub broken: Vec<RunnerFileError>,
}

pub fn runner_folder(config_home: Option<&str>, home: Option<&str>) -> PathBuf {
    config_home
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home.unwrap_or_default()).join(".config"))
        .join("pwe/runners")
}

pub fn read_runner_files(user_folder: &Path) -> RunnerFiles {
    let mut texts: BTreeMap<String, (String, Result<String, String>)> = SHIPPED
        .iter()
        .map(|(name, text)| (name.to_string(), (name.to_string(), Ok(text.to_string()))))
        .collect();
    for file in user_runner_files(user_folder) {
        let Some(name) = file.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let text = fs::read_to_string(&file).map_err(|failure| failure.to_string());
        texts.insert(name.to_string(), (file.display().to_string(), text));
    }
    texts
        .into_values()
        .fold(RunnerFiles::default(), |mut files, (label, text)| {
            let read = match text {
                Ok(text) => read_runner_file(&label, &text),
                Err(reason) => Err(RunnerFileError::Unreadable {
                    file: label,
                    reason,
                }),
            };
            match read {
                Ok(spec) => files.runners.push(spec),
                Err(error) => files.broken.push(error),
            }
            files
        })
}

fn user_runner_files(user_folder: &Path) -> Vec<PathBuf> {
    fs::read_dir(user_folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == RUNNER_FILE_EXTENSION)
        })
        .collect()
}
