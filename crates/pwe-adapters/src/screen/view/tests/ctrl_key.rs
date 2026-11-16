use super::*;
use pwe_domain::{JumpTarget, Stroke};

#[test]
fn a_picker_on_a_ctrl_key_shows_that_key_with_a_caret() {
    let bookmark = [JumpTarget {
        action_id: "bookmark".to_string(),
        label: "bookmark".to_string(),
        directory: "bookmark picker".to_string(),
        pick_key: Stroke::Ctrl('b').pick_key().expect("a pick key"),
    }];

    let drawn = render(&browsing(&bookmark, 0), &layout(), 40, 9);

    assert!(
        without_escape_sequences(&drawn).contains("  ^B  bookmark"),
        "{drawn:?}"
    );
}
