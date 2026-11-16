use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use pwe_domain::Key;

use super::*;

fn name(code: KeyCode, modifiers: KeyModifiers) -> Option<String> {
    key_name(KeyEvent::new(code, modifiers))
}

fn named(code: KeyCode, modifiers: KeyModifiers) -> String {
    name(code, modifiers).unwrap_or_default()
}

#[test]
fn ctrl_enter_reaches_rows_that_claim_ctrl_enter() {
    let pressed = named(KeyCode::Enter, KeyModifiers::CONTROL);

    assert_eq!(pressed, "ctrl-enter");
    assert_eq!(Key::parse(&pressed), Key::parse("ctrl-enter"));
    assert_eq!(named(KeyCode::Enter, KeyModifiers::NONE), "enter");
}

#[test]
fn ctrl_semicolon_is_its_own_key() {
    assert_eq!(named(KeyCode::Char(';'), KeyModifiers::CONTROL), "ctrl-;");
    assert_eq!(named(KeyCode::Char(';'), KeyModifiers::NONE), ";");
    assert_eq!(named(KeyCode::Char(';'), KeyModifiers::ALT), "alt-;");
}

#[test]
fn ctrl_i_and_tab_are_different_keys() {
    assert_eq!(named(KeyCode::Char('i'), KeyModifiers::CONTROL), "ctrl-i");
    assert_eq!(named(KeyCode::Tab, KeyModifiers::NONE), "tab");
    assert_eq!(named(KeyCode::BackTab, KeyModifiers::SHIFT), "shift-tab");
}

#[test]
fn a_released_key_is_no_key() {
    let released = KeyEvent::new_with_kind(
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    );
    let repeated =
        KeyEvent::new_with_kind(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Repeat);

    assert_eq!(key_name(released), None);
    assert_eq!(key_name(repeated).as_deref(), Some("a"));
}

#[test]
fn ctrl_alt_letters_keep_both_modifiers() {
    let both = KeyModifiers::CONTROL | KeyModifiers::ALT;

    assert_eq!(named(KeyCode::Char('x'), both), "ctrl-alt-x");
}

#[test]
fn a_letter_keeps_its_case_and_a_ctrl_letter_is_lower_case() {
    assert_eq!(named(KeyCode::Char('D'), KeyModifiers::SHIFT), "D");
    assert_eq!(named(KeyCode::Char('U'), KeyModifiers::CONTROL), "ctrl-u");
    assert_eq!(named(KeyCode::Char('b'), KeyModifiers::ALT), "alt-b");
}

#[test]
fn space_is_spelled_as_a_word() {
    assert_eq!(named(KeyCode::Char(' '), KeyModifiers::NONE), "space");
    assert_eq!(
        named(KeyCode::Char(' '), KeyModifiers::CONTROL),
        "ctrl-space"
    );
}

#[test]
fn named_keys_are_spelled_as_row_keys_spell_them() {
    let spelled = [
        (KeyCode::Esc, "esc"),
        (KeyCode::Backspace, "bspace"),
        (KeyCode::Delete, "del"),
        (KeyCode::Up, "up"),
        (KeyCode::Down, "down"),
        (KeyCode::Left, "left"),
        (KeyCode::Right, "right"),
        (KeyCode::Home, "home"),
        (KeyCode::End, "end"),
        (KeyCode::PageUp, "page-up"),
        (KeyCode::PageDown, "page-down"),
        (KeyCode::Insert, "insert"),
        (KeyCode::F(1), "f1"),
        (KeyCode::F(12), "f12"),
    ];

    for (code, spelling) in spelled {
        assert_eq!(named(code, KeyModifiers::NONE), spelling);
    }
}

#[test]
fn a_named_key_carries_its_modifiers() {
    assert_eq!(named(KeyCode::Up, KeyModifiers::SHIFT), "shift-up");
    assert_eq!(named(KeyCode::Backspace, KeyModifiers::ALT), "alt-bspace");
    assert_eq!(named(KeyCode::F(2), KeyModifiers::NONE), "f2");
    assert!(Key::parse(&named(KeyCode::F(2), KeyModifiers::NONE)).is_some());
}

#[test]
fn a_key_with_no_spelling_is_no_key() {
    assert_eq!(name(KeyCode::CapsLock, KeyModifiers::NONE), None);
}
