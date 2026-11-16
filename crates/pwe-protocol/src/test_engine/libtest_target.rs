use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

pub(super) enum TestTarget {
    Unknown,
    File(String),
    DocTests,
}

pub(super) struct TestTargets {
    repo_root: String,
    sources: HashMap<String, String>,
    tests_started: bool,
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    target: Option<ArtifactTarget>,
    executable: Option<String>,
}

#[derive(Deserialize)]
struct ArtifactTarget {
    src_path: String,
}

const COMPILER_ARTIFACT: &str = "compiler-artifact";

impl TestTargets {
    pub(super) fn new(repo_root: &str) -> Self {
        TestTargets {
            repo_root: repo_root.to_string(),
            sources: HashMap::new(),
            tests_started: false,
        }
    }

    pub(super) fn read(&mut self, line: &str) -> Option<TestTarget> {
        if !self.tests_started && line.starts_with('{') {
            self.record_artifact(line);
            return None;
        }
        let line = line.trim();
        if line.starts_with("Doc-tests ") {
            self.tests_started = true;
            return Some(TestTarget::DocTests);
        }
        let running = line.strip_prefix("Running ")?;
        let running = running.strip_prefix("unittests ").unwrap_or(running);
        let (printed_file, executable) = running.split_once(" (")?;
        self.tests_started = true;
        let source = executable
            .strip_suffix(')')
            .and_then(file_name)
            .and_then(|name| self.sources.get(name));
        Some(TestTarget::File(
            source.map_or(printed_file, String::as_str).to_string(),
        ))
    }

    fn record_artifact(&mut self, line: &str) {
        let Ok(message) = serde_json::from_str::<CargoMessage>(line) else {
            return;
        };
        let (Some(target), Some(executable)) = (message.target, message.executable) else {
            return;
        };
        let in_repo = Path::new(&target.src_path)
            .strip_prefix(&self.repo_root)
            .ok()
            .and_then(Path::to_str);
        if let (true, Some(name), Some(source)) = (
            message.reason == COMPILER_ARTIFACT,
            file_name(&executable),
            in_repo,
        ) {
            self.sources.insert(name.to_string(), source.to_string());
        }
    }
}

fn file_name(path: &str) -> Option<&str> {
    Path::new(path).file_name()?.to_str()
}

#[cfg(test)]
mod tests;
