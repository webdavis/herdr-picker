use std::fs;

use super::*;
use crate::socket_server::{Reply, SocketServer};
use herdr_picker_application::JumpError;
use serde_json::json;

const LAYOUT: &str = r#"{"id":"req_1","result":{"type":"pane_layout","layout":{"area":{"x":0,"y":0,"width":175,"height":50}}}}"#;
const OK: &str = r#"{"id":"req_2","result":{"type":"ok"}}"#;
const NOT_LINKED: &str =
    r#"{"id":"req_2","error":{"code":"plugin_not_found","message":"plugin not found"}}"#;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-cli-popup-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn files_pane() -> PopupPane {
    PopupPane::configured_picker(
        "files",
        "/home/me/config.toml",
        "/state/picker-result-42",
        Some("/home/me/repos"),
    )
}

#[test]
fn inside_herdr_the_picker_opens_in_a_popup_and_outside_it_runs_in_place() {
    assert_eq!(
        popup_socket(Some("/home/me/.config/herdr/herdr.sock".to_string())),
        Some(PathBuf::from("/home/me/.config/herdr/herdr.sock"))
    );
    assert_eq!(popup_socket(Some(String::new())), None);
    assert_eq!(popup_socket(None), None);
}

#[test]
fn the_popup_takes_most_of_the_window_and_is_told_the_picker_its_config_and_the_result_file() {
    let server = SocketServer::start(vec![Reply::Line(LAYOUT.into()), Reply::Line(OK.into())]);

    assert_eq!(open(&server.path, files_pane()), Ok(()));
    assert_eq!(server.seen()["method"], "pane.layout");
    assert_eq!(
        server.seen()["params"],
        json!({
            "plugin_id": "herdr-picker",
            "entrypoint": "run_picker",
            "placement": "popup",
            "width": 140,
            "height": 40,
            "cwd": "/home/me/repos",
            "env": {
                "HERDR_PICKER_NAME": "files",
                "HERDR_PICKER_CONFIG": "/home/me/config.toml",
                "HERDR_PICKER_RESULT": "/state/picker-result-42"
            }
        })
    );
}

#[test]
fn a_herdr_without_the_plugin_linked_refuses_the_popup() {
    let server = SocketServer::start(vec![
        Reply::Line(LAYOUT.into()),
        Reply::Line(NOT_LINKED.into()),
    ]);

    assert_eq!(
        open(&server.path, files_pane()),
        Err(JumpError::Server {
            code: "plugin_not_found".to_string(),
            message: "plugin not found".to_string(),
        })
    );
}

#[test]
fn a_popup_asked_for_a_picker_the_config_lacks_answers_with_that_failure() {
    let sandbox = Sandbox::new("unknown");
    let config = sandbox.0.join("config.toml");
    fs::write(
        &config,
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = \"files.sh\"\n",
    )
    .expect("a config");
    let result = sandbox.0.join("picker-result");

    assert!(answer_in_popup(Some(result.clone()), "nope", &config).is_ok());
    assert_eq!(
        fs::read_to_string(&result).expect("a result"),
        "failed\nno picker is named nope"
    );
}

#[test]
fn a_popup_whose_picker_cannot_start_answers_with_that_failure() {
    let sandbox = Sandbox::new("unstartable");
    let config = sandbox.0.join("config.toml");
    fs::write(
        &config,
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = \"/nonexistent/files.sh\"\n",
    )
    .expect("a config");
    let result = sandbox.0.join("picker-result");

    assert!(answer_in_popup(Some(result.clone()), "files", &config).is_ok());
    let answer = fs::read_to_string(&result).expect("a result");
    assert!(
        answer.starts_with("failed\npicker files: cannot start /nonexistent/files.sh: "),
        "{answer}"
    );
}

#[test]
fn the_popup_side_needs_the_result_file_herdr_was_told_about() {
    let refusal = answer_in_popup(None, "files", Path::new("/unused/config.toml"));

    assert!(
        matches!(&refusal, Err(CommandError::Failed(reason)) if reason.contains("HERDR_PICKER_RESULT")),
        "{refusal:?}"
    );
}
