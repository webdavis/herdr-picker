use std::collections::VecDeque;

use herdr_picker_domain::{Action, Key, PickerRow, Preview};
use serde_json::Value;

use super::super::step;
use crate::builtin::clock::stamp;
use crate::builtin::paint::{dim, green, teal};
use crate::builtin::rows::key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Message {
    pub(super) date: String,
    pub(super) sender: String,
    pub(super) text: String,
    chats: Vec<String>,
    // The conversation it belongs to, or the one being read when BlueBubbles names none.
    chat: Option<String>,
    // Each attachment's guid and file name.
    pub(super) attachments: Vec<(String, String)>,
}

pub(super) fn message(value: &Value, chat: Option<&str>, offset: i64) -> Option<Message> {
    value["guid"].as_str()?;
    let said = |value: &Value| value.as_str().unwrap_or_default().to_string();
    let sender = match value["isFromMe"].as_bool() == Some(true) {
        true => "Me".to_string(),
        false => value["handle"]["address"]
            .as_str()
            .unwrap_or("(unknown sender)")
            .to_string(),
    };
    let listed = |field: &str| value[field].as_array().cloned().unwrap_or_default();
    let chats: Vec<String> = listed("chats")
        .iter()
        .map(|chat| said(&chat["guid"]))
        .collect();
    let attachments = listed("attachments")
        .iter()
        .filter_map(|attachment| {
            let guid = attachment["guid"].as_str()?.to_string();
            let name = attachment["transferName"]
                .as_str()
                .unwrap_or(&guid)
                .to_string();
            Some((guid, name))
        })
        .collect();
    Some(Message {
        date: value["dateCreated"]
            .as_i64()
            .map_or("(unknown date)".to_string(), |sent| {
                stamp(sent / 1000, offset)
            }),
        sender,
        text: said(&value["text"]),
        chat: chats.first().cloned().or(chat.map(str::to_string)),
        chats,
        attachments,
    })
}

// Each message with the two before it and the three after it in its conversation.
pub(super) fn with_neighbors(
    messages: impl Iterator<Item = Message>,
) -> impl Iterator<Item = (Message, String)> {
    let mut source = messages;
    let mut ahead: VecDeque<Message> = source.by_ref().take(3).collect();
    let mut before: VecDeque<Message> = VecDeque::new();
    std::iter::from_fn(move || {
        let current = ahead.pop_front()?;
        ahead.extend(source.next());
        let nearby: Vec<String> = before
            .iter()
            .chain(&ahead)
            .filter(|neighbor| neighbor.chat == current.chat)
            .map(signed)
            .collect();
        before.push_back(current.clone());
        if before.len() > 2 {
            before.pop_front();
        }
        Some((current, nearby.join("\n\n")))
    })
}

pub(super) fn history_row(message: &Message, nearby: &str) -> PickerRow {
    let summary = match (message.text.is_empty(), message.attachments.is_empty()) {
        (false, _) => &message.text,
        (true, false) => "[attachments]",
        (true, true) => "[message without text]",
    };
    let sender = match message.sender.as_str() {
        "Me" => green("Me"),
        other => teal(other),
    };
    let mut detail = detail(message);
    if !nearby.is_empty() {
        detail.push_str(&format!(
            "\n\nNearby messages in this conversation:\n{nearby}"
        ));
    }
    row(
        message,
        format!("{} {sender}: {summary}", dim(&message.date)),
        detail,
        &message.text,
    )
}

// Enter copies the value and Alt-A copies it with its date and sender.
pub(super) fn row(message: &Message, label: String, detail: String, value: &str) -> PickerRow {
    let signed = format!("{} {}\n{value}", message.date, message.sender);
    PickerRow {
        label,
        preview: Some(Preview::Text(detail)),
        enter: Action::Copy(value.to_string()),
        keys: [(key("alt-a"), Action::Copy(signed))]
            .into_iter()
            .chain(conversation(message))
            .collect(),
        group: None,
    }
}

// Alt-M opens the message's conversation.
pub(super) fn conversation(message: &Message) -> Option<(Key, Action)> {
    let chat = message.chat.clone()?;
    Some((key("alt-m"), step("message-history", vec![chat])))
}

pub(super) fn detail(message: &Message) -> String {
    let chats = message.chats.join(", ");
    format!(
        "Date: {}\nFrom: {}\nChat: {chats}\n\n{}",
        message.date, message.sender, message.text
    )
}

fn signed(message: &Message) -> String {
    format!("{} {}\n{}", message.date, message.sender, message.text)
}

#[cfg(test)]
mod tests;
