use std::path::Path;

use herdr_picker_adapters::{Picked, SEND_ROWS};
use herdr_picker_domain::{Action, PickerCall, PickerRow};

use super::super::chosen_action;

#[test]
fn alt_o_leaves_for_the_send_step_carrying_the_marked_rows_as_text() {
    let row = |label: &str| PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Nothing,
        keys: Vec::new(),
        group: None,
    };
    let picked = Picked {
        query: String::new(),
        key: SEND_ROWS.to_string(),
        rows: vec![row("\x1b[2msrc/a.rs\x1b[0m:3  broken"), row("b")],
    };

    assert_eq!(
        chosen_action(&picked, None, Path::new("/q.json"), "Broccoli Tickets"),
        Action::Next(PickerCall {
            name: "send-to".to_string(),
            args: Vec::new(),
            state: Some(
                r#"{"text":"src/a.rs:3  broken\nb","from":"Broccoli Tickets"}"#.to_string()
            ),
            query: None,
        })
    );
}
