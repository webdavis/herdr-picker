use herdr_picker_domain::{Action, PickerRow, Preview};

use super::*;
use crate::builtin::rows::key;

const LISTED: &str = "Toggle Focus (72CB90F2-3819-4432-BA00-357D762A1CC6)
Not a shortcut line
Quick (Note) (DA70C82D-FA6F-4B24-A23A-3A9C48C62ACA)
";

#[test]
fn a_shortcut_prepares_shortcuts_run_and_ctrl_o_its_editor() {
    let rows = shortcut_rows(LISTED);

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0],
        PickerRow {
            label: "Toggle Focus".to_string(),
            preview: Some(Preview::Text(
                "Identifier: 72CB90F2-3819-4432-BA00-357D762A1CC6\nEnter prepares shortcuts run. Ctrl-O opens its editor.".to_string()
            )),
            enter: Action::Insert("shortcuts run 72CB90F2-3819-4432-BA00-357D762A1CC6".to_string()),
            keys: vec![
                (key("ctrl-enter"), Action::Run("shortcuts run 72CB90F2-3819-4432-BA00-357D762A1CC6".to_string())),
                (key("ctrl-o"), Action::Insert("shortcuts view 72CB90F2-3819-4432-BA00-357D762A1CC6".to_string())),
            ],
            group: None,
        }
    );
    assert_eq!(rows[1].label, "Quick (Note)");
}
