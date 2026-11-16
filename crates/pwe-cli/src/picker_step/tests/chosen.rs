use pwe_domain::{Location, PickerRow};

use super::*;

#[test]
fn esc_goes_back_and_any_other_key_does_what_the_marked_rows_say() {
    let row = |path: &str| PickerRow {
        label: path.to_string(),
        preview: None,
        enter: Action::Edit(vec![Location {
            path: path.to_string(),
            line: None,
        }]),
        keys: Vec::new(),
        group: None,
    };
    let picked = |key: &str| Picked {
        query: String::new(),
        key: key.to_string(),
        rows: vec![row("a"), row("b")],
    };

    assert_eq!(
        chosen_action(&picked("esc"), None, Path::new("/q.json"), "Files"),
        Action::Back
    );
    assert_eq!(
        chosen_action(&picked("ctrl-o"), None, Path::new("/q.json"), "Files"),
        Action::Nothing
    );
    assert_eq!(
        chosen_action(&picked("enter"), None, Path::new("/q.json"), "Files"),
        Action::Edit(vec![
            Location {
                path: "a".to_string(),
                line: None
            },
            Location {
                path: "b".to_string(),
                line: None
            },
        ])
    );
}

#[test]
fn leaving_to_save_the_quickfix_list_opens_neovim_with_it() {
    let picked = Picked {
        query: String::new(),
        key: SAVE_QUICKFIX.to_string(),
        rows: Vec::new(),
    };

    assert_eq!(
        chosen_action(&picked, None, Path::new("/state/quickfix.json"), "Files"),
        Action::Run(
            "nvim -c 'call setqflist(json_decode(join(readfile('\\''/state/quickfix.json'\\''))))' -c copen"
                .to_string()
        )
    );
}

#[test]
fn a_source_picked_in_the_chooser_shows_the_step_again_from_that_source() {
    let picked = Picked {
        query: "hid".to_string(),
        key: SWITCH_SOURCE.to_string(),
        rows: Vec::new(),
    };

    assert_eq!(
        chosen_action(&picked, Some("hidden"), Path::new("/q.json"), "Files"),
        Action::Source("hidden".to_string())
    );
}
