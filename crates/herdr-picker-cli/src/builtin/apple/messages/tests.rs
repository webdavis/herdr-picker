use herdr_picker_domain::{Action, PickerCall, Preview};
use serde_json::json;

use super::*;

fn step(name: &str, args: &[&str]) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state: None,
        query: None,
    })
}

#[test]
fn messages_opens_on_conversations_all_messages_links_and_attachments() {
    let rows = menu_rows();

    assert_eq!(
        rows.iter()
            .map(|row| (row.label.as_str(), &row.enter))
            .collect::<Vec<_>>(),
        [
            ("Conversations", &step("chats", &[])),
            ("All messages", &step("message-history", &[])),
            ("Links", &step("message-links", &[])),
            ("Attachments", &step("message-attachments", &[])),
        ]
    );
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "Choose a conversation, then search its history.".to_string()
        ))
    );
}

#[test]
fn a_conversation_is_named_and_enter_opens_its_history() {
    let chats = [
        json!({"guid": "iMessage;+;chat1", "displayName": "Family", "participants": [{"address": "+1 (555) 010-0100"}, {"address": "Ada@Example.com"}],
               "lastMessage": {"text": "Dinner at six"}}),
        json!({"guid": "iMessage;-;bob@example.com", "displayName": "", "participants": [{"address": "bob@example.com"}]}),
    ];

    let rows = chat_rows(&chats, &[]);

    assert_eq!(rows[0].label, "Family  \u{1b}[2mDinner at six\u{1b}[0m");
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "Participant: +1 (555) 010-0100\nParticipant: Ada@Example.com\n\nDinner at six\nEnter opens this conversation's history.".to_string()
        ))
    );
    assert_eq!(
        rows[0].enter,
        step("message-history", &["iMessage;+;chat1"])
    );
    assert_eq!(rows[1].label, "bob@example.com");
    assert_eq!(
        chat_rows(&chats, &["ada@example.COM", "5550100100"])
            .iter()
            .map(|row| row.label.as_str())
            .collect::<Vec<_>>(),
        ["Family  \u{1b}[2mDinner at six\u{1b}[0m"]
    );
}
