use std::collections::HashMap;

use herdr_picker_domain::{Action, Preview};
use serde_json::json;

use super::*;
use crate::builtin::apple::messages::history::message;

#[test]
fn an_attachment_prepares_opening_its_file_or_says_it_is_unavailable() {
    let sent = json!({"guid": "m2", "isFromMe": false, "handle": {"address": "+15550100"}, "dateCreated": 1_792_411_320_000_i64,
        "text": "", "chats": [{"guid": "iMessage;-;+15550100"}],
        "attachments": [{"guid": "at1", "transferName": "photo.heic"}, {"guid": "at2"}]});
    let sent = message(&sent, None, 0).expect("a message");
    let paths = HashMap::from([(
        "at1".to_string(),
        "/Users/me/Library/Messages/Attachments/photo.heic".to_string(),
    )]);
    let rows = attachment_rows(&sent, &paths);

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].label,
        "photo.heic  \u{1b}[2m+15550100  2026-10-19 12:02\u{1b}[0m"
    );
    assert_eq!(
        rows[0].enter,
        Action::Insert("open /Users/me/Library/Messages/Attachments/photo.heic".to_string())
    );
    assert_eq!(
        rows[1].label,
        "at2  \u{1b}[2m+15550100  2026-10-19 12:02\u{1b}[0m"
    );
    assert_eq!(rows[1].enter, Action::Nothing);
    assert!(matches!(&rows[1].preview, Some(Preview::Text(text))
        if text.starts_with("File: Original attachment unavailable or Messages database access denied.\n")));
}

#[test]
fn the_attachment_files_come_from_the_messages_database_rows_that_name_one() {
    let tree = crate::builtin::tests::Tree::new(
        "attachments",
        &["home/Library/Messages/Attachments/a/photo.heic"],
    );
    let home = tree.0.join("home");
    let listed = r#"[{"guid":"at1","filename":"~/Library/Messages/Attachments/a/photo.heic"},{"guid":"at2","filename":"~/Library/Messages/Attachments/gone.png"}]"#;

    assert_eq!(
        files(listed, &home),
        HashMap::from([(
            "at1".to_string(),
            tree.path("home/Library/Messages/Attachments/a/photo.heic")
        )])
    );
}
