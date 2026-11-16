use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::KeyEvent;

use super::*;

fn press(code: KeyCode, modifiers: KeyModifiers) -> Option<PickKey> {
    keystroke_of(KeyEvent::new(code, modifiers))
}

#[test]
fn a_plain_or_shifted_character_press_is_that_character() {
    assert_eq!(
        press(KeyCode::Char('d'), KeyModifiers::NONE),
        Some(PickKey::Character('d'))
    );
    assert_eq!(
        press(KeyCode::Char('D'), KeyModifiers::SHIFT),
        Some(PickKey::Character('D'))
    );
}

#[test]
fn ctrl_and_alt_characters_are_reported_with_their_modifier() {
    assert_eq!(
        press(KeyCode::Char('u'), KeyModifiers::CONTROL),
        Some(PickKey::Control('u'))
    );
    assert_eq!(
        press(
            KeyCode::Char('U'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        ),
        Some(PickKey::Control('u'))
    );
    assert_eq!(
        press(KeyCode::Char('b'), KeyModifiers::ALT),
        Some(PickKey::Alt('b'))
    );
    assert_eq!(
        press(
            KeyCode::Char('h'),
            KeyModifiers::CONTROL | KeyModifiers::ALT
        ),
        Some(PickKey::Other),
        "ctrl+alt chords are left to herdr"
    );
}

#[test]
fn the_named_keys_are_reported_by_name() {
    for (code, key) in [
        (KeyCode::Esc, PickKey::Escape),
        (KeyCode::Enter, PickKey::Enter),
        (KeyCode::Backspace, PickKey::Backspace),
        (KeyCode::Delete, PickKey::Delete),
        (KeyCode::Up, PickKey::Up),
        (KeyCode::Down, PickKey::Down),
        (KeyCode::Left, PickKey::Left),
        (KeyCode::Right, PickKey::Right),
        (KeyCode::Home, PickKey::Home),
        (KeyCode::End, PickKey::End),
    ] {
        assert_eq!(press(code, KeyModifiers::NONE), Some(key), "{code:?}");
    }
}

#[test]
fn backspace_with_ctrl_or_alt_is_the_word_backspace() {
    assert_eq!(
        press(KeyCode::Backspace, KeyModifiers::ALT),
        Some(PickKey::WordBackspace)
    );
    assert_eq!(
        press(KeyCode::Backspace, KeyModifiers::CONTROL),
        Some(PickKey::WordBackspace)
    );
}

#[test]
fn a_key_with_no_name_here_is_other() {
    for other in [KeyCode::F(1), KeyCode::Tab, KeyCode::PageUp] {
        assert_eq!(
            press(other, KeyModifiers::NONE),
            Some(PickKey::Other),
            "{other:?}"
        );
    }
}

#[test]
fn the_release_of_a_key_is_not_a_keystroke() {
    let release = KeyEvent::new_with_kind(
        KeyCode::Char('d'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    );

    assert_eq!(keystroke_of(release), None);
    assert_eq!(
        keystroke_of(KeyEvent::new_with_kind(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            KeyEventKind::Repeat,
        )),
        Some(PickKey::Character('d')),
        "a held key still answers the menu"
    );
}

#[test]
fn a_key_press_is_a_key_and_a_resize_is_a_resize() {
    assert_eq!(
        input_of(Event::Key(KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::NONE
        ))),
        Some(PickInput::Key(PickKey::Character('d')))
    );
    assert_eq!(input_of(Event::Resize(80, 24)), Some(PickInput::Resized));
}

#[test]
fn a_focus_change_and_a_paste_are_waited_through() {
    for ignored in [
        Event::FocusGained,
        Event::FocusLost,
        Event::Paste("dotfiles".to_string()),
    ] {
        assert_eq!(input_of(ignored.clone()), None, "{ignored:?}");
    }
}

static RESTORED: AtomicBool = AtomicBool::new(false);

fn mark_restored() {
    RESTORED.store(true, Ordering::Relaxed);
}

#[test]
fn leaving_the_guards_scope_restores_the_terminal() {
    RESTORED.store(false, Ordering::Relaxed);

    drop(RawMode {
        restore: mark_restored,
    });

    assert!(
        RESTORED.load(Ordering::Relaxed),
        "the terminal was left in raw mode"
    );
}
