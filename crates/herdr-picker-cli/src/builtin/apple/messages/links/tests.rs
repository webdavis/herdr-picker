use herdr_picker_domain::Action;
use serde_json::json;

use super::*;
use crate::builtin::apple::messages::history::message;

#[test]
fn each_link_in_a_message_is_a_row_that_copies_it() {
    let sent = json!({"guid": "m3", "isFromMe": true, "dateCreated": 1_792_411_380_000_i64,
        "text": "See https://example.com/a, ok? (http://b.example/x?y=1).", "chats": [{"guid": "c"}]});
    let rows = link_rows(&message(&sent, None, 0).expect("a message"));

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].label,
        "\u{1b}[38;2;137;180;250mhttps://example.com/a\u{1b}[0m  \u{1b}[2mMe  2026-10-19 12:03\u{1b}[0m"
    );
    assert_eq!(
        rows[0].enter,
        Action::Copy("https://example.com/a".to_string())
    );
    assert_eq!(
        rows[1].enter,
        Action::Copy("http://b.example/x?y=1".to_string())
    );
}
