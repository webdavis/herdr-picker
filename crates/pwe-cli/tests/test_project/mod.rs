#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static NEXT_FOLDER: AtomicU64 = AtomicU64::new(0);
static SPAWNING: Mutex<()> = Mutex::new(());

pub struct TestProject {
    pub folder: PathBuf,
}

impl TestProject {
    pub fn new(name: &str) -> Self {
        let number = NEXT_FOLDER.fetch_add(1, Ordering::Relaxed);
        let folder =
            std::env::temp_dir().join(format!("pwe-test-{name}-{}-{number}", process::id()));
        let _ = fs::remove_dir_all(&folder);
        for part in ["project/.git", "home", "bin", "state", "config"] {
            fs::create_dir_all(folder.join(part)).unwrap_or_else(|failure| panic!("{failure}"));
        }
        let folder = folder.canonicalize().unwrap_or(folder);
        Self { folder }
    }

    pub fn project(&self) -> PathBuf {
        self.folder.join("project")
    }

    pub fn state(&self) -> PathBuf {
        self.folder.join("state")
    }

    pub fn file(&self, path: &str, text: &str) -> &Self {
        let file = self.project().join(path);
        write(&file, text);
        self
    }

    pub fn user_runner(&self, name: &str, text: &str) -> &Self {
        write(&self.folder.join("config/pwe/runners").join(name), text);
        self
    }

    pub fn fake_tool(&self, name: &str, output: &str, exit_code: i32) -> &Self {
        let replay = self.folder.join(format!("{name}.output"));
        write(
            &replay,
            &output.replace("/work", &self.project().to_string_lossy()),
        );
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat '{}'\nexit {exit_code}\n",
            self.folder.join(format!("{name}.arguments")).display(),
            replay.display()
        );
        let program = self.folder.join("bin").join(name);
        let _spawning = SPAWNING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        write(&program, &script);
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700))
            .unwrap_or_else(|failure| panic!("{failure}"));
        self
    }

    pub fn tool_arguments(&self, name: &str) -> Vec<String> {
        fs::read_to_string(self.folder.join(format!("{name}.arguments")))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    pub fn pwe(&self, words: &[&str]) -> Output {
        self.spawn(words)
            .wait_with_output()
            .unwrap_or_else(|failure| panic!("pwe did not finish: {failure}"))
    }

    pub fn spawn(&self, words: &[&str]) -> Child {
        let _spawning = SPAWNING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Command::new(env!("CARGO_BIN_EXE_pwe"))
            .args(words)
            .current_dir(self.project())
            .env("HOME", self.folder.join("home"))
            .env("XDG_STATE_HOME", self.state())
            .env("XDG_CONFIG_HOME", self.folder.join("config"))
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.folder.join("bin").display()),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|failure| panic!("pwe did not run: {failure}"))
    }
}

fn write(file: &Path, text: &str) {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|failure| panic!("{failure}"));
    }
    fs::write(file, text).unwrap_or_else(|failure| panic!("{failure}"));
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.folder);
    }
}

pub fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|failure| panic!("{failure}: {text}"))
}

pub fn exit_code(output: &Output) -> Option<i32> {
    output.status.code()
}
