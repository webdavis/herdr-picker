use herdr_picker_domain::{PickerRow, Preview};
use serde_json::Value;

use super::step;
use crate::builtin::paint::dim;

pub(super) mod attachments;
pub(super) mod history;
mod links;
mod reading;

pub(super) use reading::{messages, messages_step};

const MENU: [(&str, &str, &str); 4] = [
    (
        "chats",
        "Conversations",
        "Choose a conversation, then search its history.",
    ),
    (
        "message-history",
        "All messages",
        "Search normalized message text across conversations.",
    ),
    (
        "message-links",
        "Links",
        "Search links with their parent message and sender.",
    ),
    (
        "message-attachments",
        "Attachments",
        "Search attachments and prepare opening the original file.",
    ),
];

pub(super) fn menu_rows() -> Vec<PickerRow> {
    MENU.iter()
        .map(|(next, label, detail)| PickerRow {
            label: label.to_string(),
            preview: Some(Preview::Text(detail.to_string())),
            enter: step(next, Vec::new()),
            keys: Vec::new(),
            group: None,
        })
        .collect()
}

// The conversations, only those with one of the addresses when a contact's Alt-M names some.
pub(super) fn chat_rows(chats: &[Value], addresses: &[&str]) -> Vec<PickerRow> {
    let wanted: Vec<String> = addresses
        .iter()
        .map(|address| normalized(address))
        .collect();
    chats
        .iter()
        .filter_map(|chat| {
            let guid = chat["guid"].as_str()?;
            let people = chat["participants"].as_array().cloned().unwrap_or_default();
            let people: Vec<&str> = people
                .iter()
                .filter_map(|person| person["address"].as_str())
                .collect();
            let shared = people
                .iter()
                .any(|person| wanted.contains(&normalized(person)));
            if !wanted.is_empty() && !shared {
                return None;
            }
            let named = chat["displayName"].as_str().filter(|name| !name.is_empty());
            let label = named.map_or_else(|| people.join(", "), str::to_string);
            let label = if label.is_empty() {
                guid.to_string()
            } else {
                label
            };
            let last = chat["lastMessage"]["text"].as_str().unwrap_or_default();
            let short: String = last.chars().take(60).collect();
            let participants: Vec<String> = people
                .iter()
                .map(|person| format!("Participant: {person}"))
                .collect();
            Some(PickerRow {
                label: if short.is_empty() {
                    label
                } else {
                    format!("{label}  {}", dim(&short))
                },
                preview: Some(Preview::Text(format!(
                    "{}\n\n{last}\nEnter opens this conversation's history.",
                    participants.join("\n")
                ))),
                enter: step("message-history", vec![guid.to_string()]),
                keys: Vec::new(),
                group: None,
            })
        })
        .collect()
}

// Email addresses match whatever their case; phone numbers whatever their spacing and punctuation.
fn normalized(address: &str) -> String {
    let address = address.trim();
    match address.contains('@') {
        true => address.to_lowercase(),
        false => address
            .chars()
            .filter(|c| !c.is_whitespace() && !"().-".contains(*c))
            .collect(),
    }
}

#[cfg(test)]
mod tests;
