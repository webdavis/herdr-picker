use super::*;

fn names(keys: impl Iterator<Item = Key>) -> Vec<String> {
    keys.map(|key| key.name().to_string()).collect()
}

#[test]
fn a_row_may_claim_enter_and_the_keys_fzf_leaves_unbound() {
    for name in [
        "enter",
        "alt-enter",
        "ctrl-o",
        "ctrl-x",
        "alt-y",
        "alt-a",
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
fn keys_that_type_edit_or_steer_the_picker_are_not_a_rows_to_claim() {
    for name in [
        "a",
        "ctrl-a",
        "ctrl-e",
        "ctrl-k",
        "alt-b",
        "alt-d",
        "alt-f",
        "tab",
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

    assert_eq!(bound.len(), 2 + 7 + 23 + 11);
    assert_eq!(&bound[..4], ["enter", "alt-enter", "ctrl-o", "ctrl-r"]);
    assert_eq!(bound.last().map(String::as_str), Some("f12"));
}
