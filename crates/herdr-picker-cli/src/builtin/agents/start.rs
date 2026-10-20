use std::fs;

use herdr_picker_domain::{Action, NewPane, PickerRow, Preview};

use crate::builtin::Places;
use crate::builtin::paint::dim;

// clauth's profiles for the two accounts: personal work, and Broccoli's team account.
pub(super) const PERSONAL: &str = "main";
pub(super) const BROCCOLI: &str = "broccoli";

// Each tool on each account, started in a new pane beside this one, in this folder.
pub(super) fn start(places: &Places) -> Vec<PickerRow> {
    let mut tools: Vec<(&str, &str, String)> = vec![
        ("Claude", "personal", format!("clauth start {PERSONAL}")),
        ("Claude", "broccoli", format!("clauth start {BROCCOLI}")),
        ("Codex", "personal", "codex".to_string()),
        ("Hermes", "default", "hermes".to_string()),
    ];
    let profiles = places
        .home
        .as_ref()
        .map(|home| home.join(".hermes/profiles"));
    let mut named: Vec<String> = profiles
        .and_then(|folder| fs::read_dir(folder).ok())
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    named.sort();
    tools.extend(
        named
            .into_iter()
            .map(|profile| ("Hermes", "profile", format!("hermes -p {profile}"))),
    );
    tools
        .into_iter()
        .map(|(tool, account, command)| PickerRow {
            label: format!("{tool}  {}", dim(&format!("{account} · {command}"))),
            preview: Some(Preview::Text(format!(
                "{command}\n\nStarts in a new pane to the right, in this folder."
            ))),
            enter: Action::InPane {
                pane: NewPane::Right,
                action: Box::new(Action::Run(command)),
            },
            keys: Vec::new(),
            group: None,
        })
        .collect()
}
