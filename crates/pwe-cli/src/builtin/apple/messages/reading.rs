use std::collections::HashMap;
use std::io::Write;

use pwe_adapters::read_picker_config;
use pwe_domain::PickerRow;
use serde_json::{Value, json};

use super::attachments::{attachment_paths, attachment_rows};
use super::history::{history_row, message, with_neighbors};
use super::links::link_rows;
use super::{chat_rows, menu_rows};
use crate::builtin::apple::bluebubbles::BlueBubbles;
use crate::builtin::clock::{local_offset, now};
use crate::builtin::page::notice;
use crate::builtin::rows::RowWriter;
use crate::builtin::{Places, finder_closing_is_no_failure};
use crate::command::{CommandError, picker_config_path};

pub(in crate::builtin::apple) fn messages() -> Vec<PickerRow> {
    match connected() {
        Ok(_) => menu_rows(),
        Err(unconnected) => vec![unconnected.row()],
    }
}

pub(in crate::builtin::apple) fn messages_step(
    name: &str,
    arguments: &[&str],
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let api = match connected() {
        Ok(api) => api,
        Err(unconnected) => return finder_closing_is_no_failure(rows.row(&unconnected.row())),
    };
    if name == "chats" {
        let body = json!({"with": ["participants", "lastMessage"], "sort": "lastmessage"});
        let mut chats = Vec::new();
        let mut listed = Vec::new();
        for chat in api.query("/chat/query", &body) {
            match chat {
                Ok(chat) => chats.push(chat),
                Err(why) => listed.push(unavailable(&why)),
            }
        }
        listed.splice(0..0, chat_rows(&chats, arguments));
        return finder_closing_is_no_failure(listed.iter().try_for_each(|row| rows.row(row)));
    }
    let chat = arguments.first().copied();
    let mut body = json!({"with": ["chats", "chats.participants", "attachments"], "sort": "DESC",
        "before": now() * 1000, "convertAttachments": false});
    if let Some(chat) = chat {
        body["chatGuid"] = chat.into();
    }
    let paths = match (name, &places.home) {
        ("message-attachments", Some(home)) => attachment_paths(home),
        _ => HashMap::new(),
    };
    let offset = local_offset();
    let mut failure = None;
    let shown = {
        let found = api.query("/message/query", &body);
        let found = found.map_while(|read| read.map_err(|why| failure = Some(why)).ok());
        let found = found.filter_map(|sent: Value| message(&sent, chat, offset));
        with_neighbors(found).try_for_each(|(sent, nearby)| {
            let listed = match name {
                "message-links" => link_rows(&sent),
                "message-attachments" => attachment_rows(&sent, &paths),
                _ => vec![history_row(&sent, &nearby)],
            };
            listed.iter().try_for_each(|row| rows.row(row))
        })
    };
    finder_closing_is_no_failure(
        shown.and_then(|()| failure.map_or(Ok(()), |why| rows.row(&unavailable(&why)))),
    )
}

enum Unconnected {
    NotConfigured,
    Unavailable(String),
}

impl Unconnected {
    fn row(&self) -> PickerRow {
        match self {
            Self::NotConfigured => notice(
                "BlueBubbles is not configured",
                "Add a [pickers.apple.bluebubbles] table with url and keychain-service to pwe's config to search Messages.",
            ),
            Self::Unavailable(why) => unavailable(why),
        }
    }
}

fn connected() -> Result<BlueBubbles, Unconnected> {
    let config = read_picker_config(&picker_config_path()).ok();
    let settings = config.and_then(|config| config.bluebubbles);
    let settings = settings.ok_or(Unconnected::NotConfigured)?;
    BlueBubbles::connect(&settings).map_err(Unconnected::Unavailable)
}

fn unavailable(why: &str) -> PickerRow {
    notice("BlueBubbles unavailable or incomplete", why)
}
