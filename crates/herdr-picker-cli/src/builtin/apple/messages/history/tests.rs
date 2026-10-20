use herdr_picker_domain::{Action, PickerCall, Preview};
use serde_json::{Value, json};

use super::*;
use crate::builtin::rows::key;

// 2026-10-19 12:00:00 UTC and the minutes after it, newest first as BlueBubbles sends them.
fn sent() -> Vec<Value> {
    vec![
        json!({"guid": "m3", "isFromMe": true, "dateCreated": 1_792_411_380_000_i64, "text": "See https://example.com/a, ok?",
               "chats": [{"guid": "iMessage;-;+15550100"}], "attachments": []}),
        json!({"guid": "m2", "isFromMe": false, "handle": {"address": "+15550100"}, "dateCreated": 1_792_411_320_000_i64,
               "text": "", "chats": [{"guid": "iMessage;-;+15550100"}],
               "attachments": [{"guid": "at1", "transferName": "photo.heic"}, {"guid": "at2"}]}),
        json!({"guid": "m1", "isFromMe": false, "handle": {"address": "+15550100"}, "dateCreated": 1_792_411_260_000_i64,
               "text": "Hello", "chats": [{"guid": "iMessage;-;+15550100"}], "attachments": []}),
        json!({"guid": "m0", "isFromMe": false, "dateCreated": "soon", "text": "Elsewhere", "chats": [{"guid": "other"}]}),
    ]
}

fn messages() -> Vec<Message> {
    sent()
        .iter()
        .filter_map(|value| message(value, None, 0))
        .collect()
}

fn history(chat: &str) -> Action {
    Action::Next(PickerCall {
        name: "message-history".to_string(),
        args: vec![chat.to_string()],
        state: None,
        query: None,
    })
}

#[test]
fn a_message_shows_when_and_who_and_enter_copies_its_text() {
    let listed: Vec<_> = with_neighbors(messages().into_iter()).collect();
    let (first, context) = &listed[0];
    let row = history_row(first, context);

    assert_eq!(
        row.label,
        "\u{1b}[2m2026-10-19 12:03\u{1b}[0m \u{1b}[38;2;166;227;161mMe\u{1b}[0m: See https://example.com/a, ok?"
    );
    assert_eq!(
        row.preview,
        Some(Preview::Text(
            "Date: 2026-10-19 12:03\nFrom: Me\nChat: iMessage;-;+15550100\n\nSee https://example.com/a, ok?\n\nNearby messages in this conversation:\n2026-10-19 12:02 +15550100\n\n\n2026-10-19 12:01 +15550100\nHello".to_string()
        ))
    );
    assert_eq!(
        row.enter,
        Action::Copy("See https://example.com/a, ok?".to_string())
    );
    assert_eq!(
        row.action(&key("alt-a")),
        Some(&Action::Copy(
            "2026-10-19 12:03 Me\nSee https://example.com/a, ok?".to_string()
        ))
    );
    assert_eq!(
        row.action(&key("alt-m")),
        Some(&history("iMessage;-;+15550100"))
    );
}

#[test]
fn a_message_without_text_or_a_date_still_has_a_row() {
    let listed = messages();

    assert!(
        history_row(&listed[1], "")
            .label
            .ends_with(": [attachments]")
    );
    assert_eq!(listed[3].date, "(unknown date)");
    assert_eq!(listed[3].sender, "(unknown sender)");
}
