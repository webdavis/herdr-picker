use std::fs;
use std::path::Path;

use filetime_free::set_age;

use super::*;
use crate::builtin::tests::Tree;

mod filetime_free {
    use std::path::Path;
    use std::process::Command;

    // touch sets a fixed modification time, so the newest-first order is the test's to choose.
    pub(super) fn set_age(path: &Path, stamp: &str) {
        let touched = Command::new("touch").args(["-t", stamp]).arg(path).status();
        assert!(
            touched.is_ok_and(|status| status.success()),
            "touch {}",
            path.display()
        );
    }
}

fn write(path: &Path, lines: &[serde_json::Value]) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
    let text: String = lines.iter().map(|line| format!("{line}\n")).collect();
    fs::write(path, text).expect("a transcript");
}

#[test]
fn claude_and_codex_sessions_started_here_come_newest_first_with_what_was_typed() {
    let home = Tree::new("sessions-home", &[]);
    let repo = Path::new("/work/dotfiles");
    let claude = home.0.join(".claude/projects/-work-dotfiles/abc-123.jsonl");
    write(
        &claude,
        &[
            serde_json::json!({"type": "user", "isMeta": true, "cwd": "/work/dotfiles", "message": {"content": "<command-name>/clear</command-name>"}}),
            serde_json::json!({"type": "user", "cwd": "/work/dotfiles", "message": {"content": "Fix the flaky test\nin CI"}}),
        ],
    );
    let elsewhere = home
        .0
        .join(".claude/projects/-work-dotfiles-other/def.jsonl");
    write(
        &elsewhere,
        &[
            serde_json::json!({"type": "user", "cwd": "/work/dotfiles-other", "message": {"content": "not here"}}),
        ],
    );
    let codex = home.0.join(".codex/sessions/2026/10/19/rollout-1.jsonl");
    write(
        &codex,
        &[
            serde_json::json!({"type": "session_meta", "payload": {"id": "019a", "cwd": "/work/dotfiles/sub"}}),
            serde_json::json!({"type": "response_item", "payload": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "# AGENTS.md instructions\n..."}]}}),
            serde_json::json!({"type": "response_item", "payload": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "Why is the build slow?"}]}}),
        ],
    );
    set_age(&claude, "202610190900");
    set_age(&codex, "202610191000");

    let found: Vec<(String, String)> = past_sessions(repo, &home.0)
        .into_iter()
        .map(|session| (session.resume, session.prompt))
        .collect();

    assert_eq!(
        found,
        vec![
            (
                "codex resume 019a".to_string(),
                "Why is the build slow?".to_string()
            ),
            (
                "claude --resume abc-123".to_string(),
                "Fix the flaky test\nin CI".to_string()
            ),
        ]
    );
}

#[test]
fn enter_resumes_the_session_in_a_new_pane_beside_this_one() {
    let home = Tree::new("sessions-row", &[]);
    let mut places = home.places("unused");
    places.home = Some(home.0.clone());
    let root = places.cwd.clone();
    let encoded = super::claude_project(&home.0, &root);
    write(
        &encoded.join("s1.jsonl"),
        &[
            serde_json::json!({"type": "user", "cwd": root.to_string_lossy(), "message": {"content": "Hello"}}),
        ],
    );

    let rows = sessions(&places);

    assert_eq!(
        rows[0].enter,
        Action::InPane {
            pane: NewPane::Right,
            action: Box::new(Action::Run("claude --resume s1".to_string())),
        }
    );
    assert!(herdr_picker_domain::unstyled(&rows[0].label).starts_with("Hello  claude · "));
}
