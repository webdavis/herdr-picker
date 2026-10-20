use std::fs;

use herdr_picker_domain::Action;

use super::{DIM, RESET, agents};
use crate::builtin::git::fixture::labels;
use crate::builtin::tests::Tree;

#[test]
fn outside_herdr_the_running_sources_say_so_and_the_others_still_work() {
    let tree = Tree::new("agents-outside", &["AGENTS.md"]);
    let places = tree.places("home");

    assert_eq!(
        labels(&agents("everything", &places)),
        vec![format!("{DIM}herdr isn't running{RESET}")]
    );
    assert_eq!(
        labels(&agents("instructions", &places)),
        vec![format!("AGENTS.md  {DIM}instructions{RESET}")]
    );
    fs::remove_file(tree.0.join("AGENTS.md")).expect("removed");
}

#[test]
fn start_offers_each_tool_on_each_account_in_a_new_pane_here() {
    let tree = Tree::new("agents-start", &["home/.hermes/profiles/lara/config.yaml"]);

    let started = agents("start", &tree.places("home"));

    assert_eq!(
        started
            .iter()
            .map(|row| herdr_picker_domain::unstyled(&row.label))
            .collect::<Vec<_>>(),
        vec![
            "Claude  personal · clauth start main",
            "Claude  broccoli · clauth start broccoli",
            "Codex  personal · codex",
            "Hermes  default · hermes",
            "Hermes  profile · hermes -p lara",
        ]
    );
    assert_eq!(
        started[1].enter,
        Action::InPane {
            pane: herdr_picker_domain::NewPane::Right,
            action: Box::new(Action::Run("clauth start broccoli".to_string())),
        }
    );
}

#[test]
fn failures_lists_the_failed_saved_runs_with_their_messages() {
    let tree = Tree::new("agents-failures", &[]);
    let places = tree.places("home");
    let saved = herdr_picker_adapters::Capture {
        category: "tests".to_string(),
        cwd: tree.0.to_string_lossy().into_owned(),
        framework: "cargo".to_string(),
        argv: ["cargo", "test"].map(str::to_string).to_vec(),
        exit_code: 101,
        stdout: "src/lib.rs:4:1: assertion failed\n".to_string(),
        stderr: String::new(),
    };
    let passed = herdr_picker_adapters::Capture {
        exit_code: 0,
        category: "diagnostics".to_string(),
        ..saved.clone()
    };
    let folder = places
        .captures
        .join(tree.0.to_string_lossy().replace('/', "%"));
    fs::create_dir_all(&folder).expect("the saved folder");
    fs::write(
        folder.join("tests-cargo.json"),
        serde_json::to_string(&saved).expect("json"),
    )
    .expect("saved");
    fs::write(
        folder.join("diagnostics-cargo.json"),
        serde_json::to_string(&passed).expect("json"),
    )
    .expect("saved");

    assert_eq!(
        labels(&agents("failures", &places)),
        vec![
            format!("{DIM}tests cargo: exit 101{RESET}"),
            "src/lib.rs:4:1: assertion failed".to_string(),
        ]
    );
}

#[test]
fn a_hermes_profile_with_shell_characters_in_its_name_is_started_quoted() {
    let tree = Tree::new(
        "agents-start-quoted",
        &["home/.hermes/profiles/my lab;x/config.yaml"],
    );

    let started = agents("start", &tree.places("home"));

    assert_eq!(
        started.last().map(|row| &row.enter),
        Some(&Action::InPane {
            pane: herdr_picker_domain::NewPane::Right,
            action: Box::new(Action::Run("hermes -p 'my lab;x'".to_string())),
        })
    );
}
