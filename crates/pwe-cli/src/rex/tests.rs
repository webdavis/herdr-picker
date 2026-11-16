use pwe_adapters::RexSettings;
use serde_json::{Value, json};

use super::*;

fn facts(label: &str, program: &str) -> String {
    json!({
        "block": "block:2",
        "view": {"session_id": "session:1", "windows": [{"layers": [{
            "bounds": [0, 0, 1, 1],
            "layout": {"block_id": "block:2", "rect": [0, 0, 1, 1]},
            "blocks": [{"block_id": "block:2", "label": label}]
        }]}]},
        "process": {"foreground": {"name": program, "cwd": "/work/repo"}},
        "size": {"columns": 80, "rows": 24}
    })
    .to_string()
}

fn answered(label: &str, program: &str) -> Option<Value> {
    answer_key(
        "ctrl-t",
        &facts(label, program),
        &RexSettings::default(),
        "/bin/pwe",
    )
    .expect("an answer")
    .map(|request| serde_json::from_str(&request).expect("a JSON request"))
}

#[test]
fn a_key_over_neovim_or_inside_pwes_float_prints_nothing() {
    assert_eq!(answered("shell", "nvim"), None);
    assert_eq!(answered("pwe", "fzf"), None);
}

#[test]
fn a_key_over_a_shell_asks_rex_for_a_float_running_the_launcher_over_that_block() {
    let request = answered("shell", "bash").expect("a float");

    assert_eq!(request["session_id"], "session:1");
    assert_eq!(request["focus"], true);
    let block = &request["layout"]["block"];
    assert_eq!(block["flavor"], "com.superlogical.terminal.shell");
    assert_eq!(block["label"], "pwe");
    assert_eq!(block["options"]["cwd"], "/work/repo");
    assert_eq!(
        block["options"]["command"],
        json!(["/bin/pwe", "launcher", "ctrl-t", "--under", "block:2"])
    );
}

#[test]
fn a_base_that_is_not_one_key_or_facts_pwe_cannot_read_are_refused() {
    let settings = RexSettings::default();

    assert!(matches!(
        answer_key("ctrl-t f", &facts("shell", "bash"), &settings, "/bin/pwe"),
        Err(CommandError::Usage)
    ));
    assert!(matches!(
        answer_key("ctrl-t", "{}", &settings, "/bin/pwe"),
        Err(CommandError::Failed(_))
    ));
}
