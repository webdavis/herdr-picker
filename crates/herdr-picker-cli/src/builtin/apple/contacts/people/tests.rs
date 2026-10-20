use std::fs;

use herdr_picker_adapters::save_contacts;

use super::*;
use crate::builtin::tests::Tree;

const BOOK: &str = "home/Library/Application Support/AddressBook";

#[test]
fn the_saved_address_book_is_read_while_its_database_is_unchanged() {
    let tree = Tree::new(
        "contacts-saved",
        &[
            &format!("{BOOK}/AddressBook-v22.abcddb"),
            &format!("{BOOK}/Sources/A1/AddressBook-v22.abcddb-wal"),
        ],
    );
    let places = tree.places("home");
    let changed = address_book_changed(&tree.0.join("home")).expect("a change time");
    save_contacts(
        &places.contacts,
        &changed,
        r#"[{"id": "P1", "name": "Ada", "fields": []}]"#,
    )
    .expect("saved");

    let found = people(&places).expect("people");

    assert_eq!(
        found
            .iter()
            .map(|person| person.name.as_str())
            .collect::<Vec<_>>(),
        ["Ada"]
    );
}

#[test]
fn the_address_books_change_time_follows_its_newest_database_file() {
    let tree = Tree::new(
        "contacts-changed",
        &[
            &format!("{BOOK}/AddressBook-v22.abcddb"),
            &format!("{BOOK}/notes.txt"),
        ],
    );
    let home = tree.0.join("home");
    let before = address_book_changed(&home).expect("a change time");

    std::thread::sleep(std::time::Duration::from_millis(5));
    let source = tree.0.join(format!("{BOOK}/Sources/A1"));
    fs::create_dir_all(&source).expect("a source folder");
    fs::write(source.join("AddressBook-v22.abcddb"), "x").expect("a source database");

    assert_ne!(address_book_changed(&home), Some(before));
    assert_eq!(address_book_changed(&tree.0.join("nobody")), None);
}
