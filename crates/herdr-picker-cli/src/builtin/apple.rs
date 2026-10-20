use std::io::Write;
use std::time::Duration;

use herdr_picker_domain::{Action, PickerCall};

use super::clock::local_offset;
use super::page::notice;
use super::program::run;
use super::rows::RowWriter;
use super::{Asked, Places, written};
use crate::command::CommandError;

mod bluebubbles;
mod contacts;
mod messages;
mod shortcuts;

use contacts::people::people;
use contacts::{call_rows, contact_rows, field_rows};
use messages::{messages, messages_step};
use shortcuts::shortcut_rows;

const PRIVACY: &str = "Check app access in Privacy & Security and dismiss pending app dialogs.";

// Messages, Contacts and Shortcuts, which only a Mac has.
pub(super) fn apple(
    source: &str,
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let listed = match (cfg!(target_os = "macos"), source) {
        (false, _) => vec![notice(
            "Apple apps need macOS",
            "Messages, Contacts and Shortcuts are read through macOS's own tools.",
        )],
        (true, "contacts") => people(places).map_or_else(
            |why| vec![notice("Contacts unavailable", &format!("{why}\n{PRIVACY}"))],
            |people| {
                let mode = asked
                    .state
                    .and_then(|state| serde_json::from_str::<String>(state).ok());
                contact_rows(&people, mode.as_deref().unwrap_or("all"), local_offset())
            },
        ),
        (true, "shortcuts") => run(
            &["shortcuts", "list", "--show-identifiers"],
            Duration::from_secs(10),
        )
        .map_or_else(
            |why| {
                vec![notice(
                    "Shortcuts unavailable",
                    &format!("{why}\n{PRIVACY}"),
                )]
            },
            |listed| shortcut_rows(&listed),
        ),
        (true, _) => messages(),
    };
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

// The apple picker's steps, or None for a name that is not one of them.
pub(super) fn apple_step(
    name: &str,
    arguments: &[&str],
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Option<Result<(), CommandError>> {
    let fields = match name {
        "contact-fields" => field_rows,
        "contact-call" => call_rows,
        "chats" | "message-history" | "message-links" | "message-attachments" => {
            return Some(messages_step(name, arguments, places, rows));
        }
        _ => return None,
    };
    let listed = people(places).map_or_else(
        |why| vec![notice("Contacts unavailable", &format!("{why}\n{PRIVACY}"))],
        |people| fields(&people, arguments, local_offset()),
    );
    Some(written(listed.iter().try_for_each(|row| rows.row(row))))
}

// A next to one of the apple picker's steps.
fn step(name: &str, args: Vec<String>) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args,
        state: None,
        query: None,
    })
}
