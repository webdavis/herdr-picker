use super::*;

fn names(keys: impl Iterator<Item = Key>) -> Vec<String> {
    keys.map(|key| key.name().to_string()).collect()
}

#[test]
fn a_row_may_claim_enter_and_any_ctrl_or_alt_letter_the_engine_does_not_need() {
    for name in [
        "enter",
        "alt-enter",
        "ctrl-o",
        "ctrl-x",
        "ctrl-e",
        "ctrl-f",
        "ctrl-l",
        "ctrl-z",
        "alt-y",
        "alt-a",
        "alt-b",
        "alt-d",
        "alt-0",
        "alt-7",
        "f2",
        "f12",
    ] {
        assert_eq!(
            Key::parse(name).map(|key| key.name().to_string()),
            Some(name.to_string())
        );
    }
}

#[test]
fn keys_that_type_or_steer_the_picker_are_not_a_rows_to_claim() {
    for name in [
        "a",
        "ctrl-c",
        "ctrl-h",
        "ctrl-i",
        "ctrl-m",
        "ctrl-n",
        "ctrl-p",
        "ctrl-q",
        "alt-o",
        "tab",
        "shift-tab",
        "up",
        "down",
        "esc",
        "ctrl-space",
        "f1",
        "ctrl-enter-x",
        "",
    ] {
        assert_eq!(Key::parse(name), None, "{name}");
    }
}

#[test]
fn ctrl_enter_is_the_alt_enter_fzf_receives_and_is_shown_as_ctrl_enter() {
    let key = Key::parse("ctrl-enter").expect("a key");

    assert_eq!(key, Key::parse("alt-enter").expect("a key"));
    assert_eq!(key.name(), "alt-enter");
    assert_eq!(key.label(), "ctrl-enter");
    assert_eq!(Key::enter().label(), "enter");
}

#[test]
fn every_claimable_key_is_bound_enter_first() {
    let bound = names(bindable_keys());

    assert_eq!(bound.len(), 2 + 19 + 25 + 10 + 2 + 11);
    assert_eq!(&bound[..4], ["enter", "alt-enter", "ctrl-a", "ctrl-b"]);
    assert_eq!(bound.last().map(String::as_str), Some("f12"));
}

#[test]
fn alt_dot_is_a_key_a_row_may_claim() {
    assert_eq!(
        Key::parse("alt-.").map(|key| key.label().to_string()),
        Some("alt-.".to_string())
    );
}

#[test]
fn alt_semicolon_is_a_key_a_row_may_claim() {
    assert_eq!(
        Key::parse("alt-;").map(|key| key.label().to_string()),
        Some("alt-;".to_string())
    );
}
