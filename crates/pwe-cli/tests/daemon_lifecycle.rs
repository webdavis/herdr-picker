use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};

use serde_json::{Value, json};

struct TempHome(PathBuf);

impl TempHome {
    fn new(name: &str) -> Self {
        let home = std::env::temp_dir().join(format!("pwc-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(&home).unwrap_or_else(|failure| panic!("{failure}"));
        Self(home)
    }

    fn pwe(&self, words: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_pwe"))
            .args(words)
            .env("HOME", &self.0)
            .env_remove("XDG_RUNTIME_DIR")
            .env_remove("XDG_STATE_HOME")
            .env_remove("XDG_CONFIG_HOME")
            .output()
            .unwrap_or_else(|failure| panic!("pwe did not run: {failure}"))
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = self.pwe(&["daemon", "stop"]);
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn succeeded(output: Output) -> String {
    assert!(
        output.status.success(),
        "pwe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|failure| panic!("{failure}: {text}"))
}

#[test]
fn start_then_status_says_running_and_not_enabled() {
    let home = TempHome::new("start");

    succeeded(home.pwe(&["daemon", "start"]));
    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--output", "json"]),
    ));

    assert_eq!(status["running"], json!(true));
    assert_eq!(status["enabled"], json!(false));
    assert_eq!(status["listens"], json!("this-machine-only"));
    assert!(status["pid"].is_u64(), "{status}");
}

#[test]
fn stop_then_status_says_not_running_in_plain_words() {
    let home = TempHome::new("stop");
    succeeded(home.pwe(&["daemon", "start"]));

    succeeded(home.pwe(&["daemon", "stop"]));
    let status = succeeded(home.pwe(&["daemon", "status"]));

    assert!(status.contains("not running"), "{status}");
    assert!(status.contains("not enabled"), "{status}");
    assert!(status.contains("this machine only"), "{status}");
}

#[test]
fn starting_twice_keeps_the_one_daemon() {
    let home = TempHome::new("twice");
    succeeded(home.pwe(&["daemon", "start"]));
    let first = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--output", "json"]),
    ));

    let again = succeeded(home.pwe(&["daemon", "start"]));
    let second = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--output", "json"]),
    ));

    assert!(again.contains("already running"), "{again}");
    assert_eq!(first["pid"], second["pid"]);
}
