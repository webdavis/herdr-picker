use pwe_domain::finder::{Ending, Found};
use pwe_domain::{Key, Location, PickerRow};

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
    let picked = |ending: Ending| Found {
        query: String::new(),
        ending,
        rows: vec![row("a"), row("b")],
    };
    let key = |name: &str| Ending::Key(Key::parse(name).expect("a row key"));

    assert_eq!(
        chosen_action(&picked(Ending::Back), None, Path::new("/q.json"), "Files"),
        Action::Back
    );
    assert_eq!(
        chosen_action(&picked(key("ctrl-o")), None, Path::new("/q.json"), "Files"),
        Action::Nothing
    );
    assert_eq!(
        chosen_action(&picked(key("enter")), None, Path::new("/q.json"), "Files"),
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
    let picked = Found {
        query: String::new(),
        ending: Ending::SaveQuickfix,
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
    let picked = Found {
        query: "hid".to_string(),
        ending: Ending::SwitchSource,
        rows: Vec::new(),
    };

    assert_eq!(
        chosen_action(&picked, Some("hidden"), Path::new("/q.json"), "Files"),
        Action::Source("hidden".to_string())
    );
}
