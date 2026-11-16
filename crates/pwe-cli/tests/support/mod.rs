#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{self, Command, Output};

use serde_json::Value;

pub struct TempHome {
    pub folder: PathBuf,
    path_variable: Option<String>,
}

const FAKE_PROGRAM: &str = r#"#!/bin/sh
printf '%s\n' "$*" >> CALLS
case "$*" in
  bootstrap*|*"enable --now"*) exec PWE daemon start >/dev/null ;;
  bootout*|*"disable --now"*) exec PWE daemon stop >/dev/null ;;
esac
"#;

impl TempHome {
    pub fn new(name: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("pwc-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap_or_else(|failure| panic!("{failure}"));
        Self {
            folder,
            path_variable: None,
        }
    }

    pub fn with_fake_launchctl_and_systemctl(mut self) -> Self {
        let bin = self.folder.join("fake-bin");
        fs::create_dir_all(&bin).unwrap_or_else(|failure| panic!("{failure}"));
        let script = FAKE_PROGRAM
            .replace("CALLS", &format!("'{}'", self.calls_file().display()))
            .replace("PWE", &format!("'{}'", env!("CARGO_BIN_EXE_pwe")));
        for name in ["launchctl", "systemctl"] {
            let program = bin.join(name);
            fs::write(&program, &script).unwrap_or_else(|failure| panic!("{failure}"));
            fs::set_permissions(&program, fs::Permissions::from_mode(0o700))
                .unwrap_or_else(|failure| panic!("{failure}"));
        }
        self.path_variable = Some(format!("{}:/usr/bin:/bin", bin.display()));
        self
    }

    pub fn with_fake_program(mut self, name: &str, script: &str) -> Self {
        let bin = self.folder.join("fake-bin");
        fs::create_dir_all(&bin).unwrap_or_else(|failure| panic!("{failure}"));
        let program = bin.join(name);
        fs::write(&program, script).unwrap_or_else(|failure| panic!("{failure}"));
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700))
            .unwrap_or_else(|failure| panic!("{failure}"));
        self.path_variable = Some(format!("{}:/usr/bin:/bin", bin.display()));
        self
    }

    fn calls_file(&self) -> PathBuf {
        self.folder.join("system-calls")
    }

    pub fn system_calls(&self) -> Vec<String> {
        fs::read_to_string(self.calls_file())
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    pub fn pwe(&self, words: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_pwe"));
        command
            .args(words)
            .env("HOME", &self.folder)
            .env_remove("XDG_RUNTIME_DIR")
            .env_remove("XDG_STATE_HOME")
            .env_remove("XDG_CONFIG_HOME");
        if let Some(path_variable) = &self.path_variable {
            command.env("PATH", path_variable);
        }
        command
            .output()
            .unwrap_or_else(|failure| panic!("pwe did not run: {failure}"))
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = self.pwe(&["daemon", "stop"]);
        let _ = fs::remove_dir_all(&self.folder);
    }
}

pub fn succeeded(output: Output) -> String {
    assert!(
        output.status.success(),
        "pwe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|failure| panic!("{failure}: {text}"))
}
