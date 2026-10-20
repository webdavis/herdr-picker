use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use herdr_picker_adapters::{save_contacts, saved_contacts};
use serde_json::json;

use super::Person;
use crate::builtin::Places;
use crate::builtin::automation::automation;

// Everyone in Contacts: the saved copy while the address book is unchanged, else asked anew.
pub(in crate::builtin::apple) fn people(places: &Places) -> Result<Vec<Person>, String> {
    let changed = places.home.as_deref().and_then(address_book_changed);
    let saved = changed
        .as_deref()
        .and_then(|changed| saved_contacts(&places.contacts, changed));
    let listed = match saved {
        Some(saved) => saved,
        None => {
            let asked = automation(&json!({"action": "contacts"}), Duration::from_secs(60))?;
            let listed = asked.to_string();
            if let Some(changed) = &changed {
                let _ = save_contacts(&places.contacts, changed, &listed);
            }
            listed
        }
    };
    serde_json::from_str(&listed)
        .map_err(|_| "Contacts answered with no list of people".to_string())
}

// When the address book's database files last changed. None when none can be seen, since then a
// saved copy could never be known to be stale.
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
