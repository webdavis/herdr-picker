use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use pwe_adapters::{save_contacts, saved_contacts};
use serde_json::json;

use super::Person;
use crate::builtin::Places;
use crate::builtin::automation::automation;

pub(in crate::builtin::apple) fn people(places: &Places) -> Result<Vec<Person>, String> {
    let changed = places.home.as_deref().and_then(address_book_changed);
    let saved = changed
        .as_deref()
        .and_then(|changed| saved_contacts(&places.contacts, changed));
    let listed = match saved {
        Some(saved) => saved,
        None => {
            let listed = ask_contacts_for_everyone_which_takes_seconds()?;
            if let Some(changed) = &changed {
                let _ = save_contacts(&places.contacts, changed, &listed);
            }
            listed
        }
    };
    serde_json::from_str(&listed)
        .map_err(|_| "Contacts answered with no list of people".to_string())
}

fn ask_contacts_for_everyone_which_takes_seconds() -> Result<String, String> {
    let asked = automation(&json!({"action": "contacts"}), Duration::from_secs(60))?;
    Ok(asked.to_string())
}

pub(super) fn address_book_changed(home: &Path) -> Option<String> {
    let book = home.join("Library/Application Support/AddressBook");
    let listed = |folder: &Path| fs::read_dir(folder).into_iter().flatten().flatten();
    let sources = listed(&book.join("Sources")).map(|source| source.path());
    std::iter::once(book.clone())
        .chain(sources)
        .flat_map(|folder| listed(&folder).collect::<Vec<_>>())
        .filter(|file| file.file_name().to_string_lossy().contains(".abcddb"))
        .filter_map(|file| file.metadata().ok()?.modified().ok())
        .max()
        .and_then(|newest| newest.duration_since(UNIX_EPOCH).ok())
        .map(|since| since.as_nanos().to_string())
}

#[cfg(test)]
mod tests;
