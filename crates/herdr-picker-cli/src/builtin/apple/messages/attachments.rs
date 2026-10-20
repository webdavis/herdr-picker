use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use herdr_picker_domain::{Action, PickerRow, Preview};
use serde::Deserialize;

use super::history::{Message, conversation, detail};
use crate::builtin::page::words;
use crate::builtin::paint::dim;
use crate::builtin::program::run;
use crate::builtin::rows::key;

// Enter prepares opening the original file and Ctrl-Enter opens it now.
pub(super) fn attachment_rows(
    message: &Message,
    paths: &HashMap<String, String>,
) -> Vec<PickerRow> {
    let sent = dim(&format!("{}  {}", message.sender, message.date));
    message
        .attachments
        .iter()
        .map(|(guid, name)| {
            let path = paths.get(guid);
            let file = path.map_or(
                "Original attachment unavailable or Messages database access denied.",
                String::as_str,
            );
            let open = path.map(|path| words(&["open", path]));
            PickerRow {
                label: format!("{name}  {sent}"),
                preview: Some(Preview::Text(format!("File: {file}\n{}", detail(message)))),
                enter: open.clone().map_or(Action::Nothing, Action::Insert),
                keys: open
                    .map(|open| (key("ctrl-enter"), Action::Run(open)))
                    .into_iter()
                    .chain(conversation(message))
                    .collect(),
                group: None,
            }
        })
        .collect()
}

#[derive(Deserialize)]
struct Attachment {
    guid: String,
    filename: String,
}

// BlueBubbles names an attachment by its guid; the Messages database knows its file, when the
// terminal has Full Disk Access.
pub(super) fn attachment_paths(home: &Path) -> HashMap<String, String> {
    let database = home.join("Library/Messages/chat.db");
    let query = "SELECT guid, filename FROM attachment WHERE filename IS NOT NULL";
    let argv = [
        "sqlite3",
        "-readonly",
        "-json",
        &database.to_string_lossy(),
        query,
    ];
    run(&argv, Duration::from_secs(10))
        .map_or_else(|_| HashMap::new(), |listed| files(&listed, home))
}

pub(super) fn files(listed: &str, home: &Path) -> HashMap<String, String> {
    let attachments: Vec<Attachment> = serde_json::from_str(listed).unwrap_or_default();
    attachments
        .into_iter()
        .filter_map(|attachment| {
            let file = match attachment.filename.strip_prefix("~/") {
                Some(inside) => home.join(inside),
                None => attachment.filename.into(),
            };
            file.is_file()
                .then(|| (attachment.guid, file.to_string_lossy().into_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests;
