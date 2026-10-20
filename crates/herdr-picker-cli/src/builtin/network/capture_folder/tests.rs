use herdr_picker_domain::Action;

use super::*;
use crate::builtin::tests::Tree;

#[test]
fn the_folder_typed_or_else_this_one_is_searched_for_captures() {
    let tree = Tree::new("capture-folder", &["traces/a.pcap", "notes.txt"]);

    let (prompt, here) = folder_rows("", &tree.0, None);
    assert_eq!(
        prompt,
        "Type the folder to search for captures, then press Enter."
    );
    assert_eq!(here.label, format!("Search {}", tree.0.display()));
    assert!(
        matches!(&here.enter, Action::Next(call) if call.name == "captures-in" && call.args == [tree.0.to_string_lossy()])
    );

    let (_, typed) = folder_rows("traces", &tree.0, None);
    assert!(matches!(&typed.enter, Action::Next(call) if call.args == [tree.path("traces")]));
    let (_, home) = folder_rows("~/traces", Path::new("/elsewhere"), Some(&tree.0));
    assert!(matches!(&home.enter, Action::Next(call) if call.args == [tree.path("traces")]));

    let (_, file) = folder_rows("notes.txt", &tree.0, None);
    assert_eq!(
        file.label,
        "\u{1b}[2mCapture search directory does not exist\u{1b}[0m"
    );
}
