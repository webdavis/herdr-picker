use herdr_picker_adapters::{open_quickfix, save_quickfix};
use herdr_picker_domain::{Action, Key, Location, QuickfixEntry};

use crate::builtin::git::fixture::{labels, read};
use crate::builtin::tests::{Tree, printed};

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

#[test]
fn the_saved_list_reopens_neovim_at_the_entry_picked_or_edits_it() {
    let tree = Tree::new("quickfix-list", &["src/a.rs"]);
    let places = tree.places("home");
    let entries = [
        ("src/a.rs", 7, "error: oops"),
        ("/elsewhere/b.rs", 2, "note"),
    ]
    .map(|(path, line, text)| QuickfixEntry {
        path: match path.starts_with('/') {
            true => path.to_string(),
            false => tree.path(path),
        },
        line,
        text: text.to_string(),
    });
    save_quickfix(&places.quickfix, &entries).expect("a saved list");

    let (_, rows) = read(&printed("tests", &[], Some("quickfix"), &places).expect("rows"));

    assert_eq!(
        labels(&rows),
        vec!["src/a.rs:7  error: oops", "/elsewhere/b.rs:2  note"]
    );
    assert_eq!(
        rows[1].enter,
        Action::Run(open_quickfix(&places.quickfix, Some(2)))
    );
    assert_eq!(
        rows[0].action(&Key::parse("ctrl-enter").expect("a key")),
        Some(&Action::Edit(vec![Location {
            path: tree.path("src/a.rs"),
            line: Some(7)
        }]))
    );
}

#[test]
fn without_a_saved_list_one_row_says_how_to_save_one() {
    let tree = Tree::new("quickfix-none", &[]);

    let (_, rows) =
        read(&printed("tests", &[], Some("quickfix"), &tree.places("home")).expect("rows"));

    assert_eq!(labels(&rows), vec![format!("{DIM}No saved results{RESET}")]);
}
