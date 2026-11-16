use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;

fn sandbox(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("hwj-contacts-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    directory
}

#[test]
fn saved_contacts_are_read_back_while_the_address_book_is_unchanged() {
    let file = sandbox("same").join("state/contacts.json");

    save_contacts(&file, "1700000000.5", r#"[{"id":"a"}]"#).expect("saved");

    assert_eq!(
        saved_contacts(&file, "1700000000.5"),
        Some(r#"[{"id":"a"}]"#.to_string())
    );
    assert_eq!(saved_contacts(&file, "1700000099.0"), None);
    let mode = fs::metadata(&file).expect("the file").permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn without_a_saved_file_there_are_no_saved_contacts() {
    assert_eq!(
        saved_contacts(&sandbox("none").join("contacts.json"), "1"),
        None
    );
}
