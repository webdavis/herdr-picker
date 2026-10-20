use herdr_picker_domain::Action;

use super::*;
use crate::builtin::tests::Tree;

fn labels(rows: &[PickerRow]) -> Vec<String> {
    rows.iter()
        .map(|row| row.label.split("  ").next().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn captures_under_the_folder_list_by_path_skipping_hidden_and_build_folders() {
    let tree = Tree::new(
        "captures",
        &[
            "a.pcap",
            "sub/d.CAP",
            ".hidden/b.pcapng",
            "node_modules/c.pcap",
            "x.txt",
        ],
    );

    let rows = captures(&tree.0, Shown::default());

    assert_eq!(labels(&rows), ["a.pcap", "sub/d.CAP"]);
    assert_eq!(rows[0].label, "a.pcap  \u{1b}[2m1 bytes\u{1b}[0m");
    assert!(
        matches!(&rows[0].enter, Action::Next(call) if call.name == "capture-analysis" && call.args == [tree.path("a.pcap")])
    );
    assert_eq!(
        rows[0].keys[0],
        (
            key("alt-."),
            Action::State(r#"{"hidden":true,"sort":false}"#.to_string())
        )
    );
    assert_eq!(
        rows[0].keys[1],
        (
            key("ctrl-t"),
            Action::State(r#"{"hidden":false,"sort":true}"#.to_string())
        )
    );

    let hidden = captures(&tree.0, Shown::from_state(Some(r#"{"hidden":true}"#)));
    assert_eq!(labels(&hidden), [".hidden/b.pcapng", "a.pcap", "sub/d.CAP"]);
}

#[test]
fn without_captures_one_row_says_so_and_still_offers_the_toggles() {
    let tree = Tree::new("no-captures", &["x.txt"]);

    let rows = captures(&tree.0, Shown::default());

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].label,
        "\u{1b}[2mNo captures under this folder\u{1b}[0m"
    );
    assert_eq!(rows[0].keys.len(), 2);
}

#[test]
fn sizes_group_their_thousands() {
    assert_eq!(grouped(1_234_567), "1,234,567");
    assert_eq!(grouped(999), "999");
}
