use std::io;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use herdr_workspace_jump_domain::PickKey;

/// Raw mode for as long as this value lives.
///
/// The popup borrows the terminal herdr gave it, so raw mode is restored on the
/// way out of every branch, including a failed read and an unwinding panic.
struct RawMode {
    restore: fn(),
}

impl Drop for RawMode {
    fn drop(&mut self) {
        (self.restore)();
    }
}

fn restore_terminal() {
    let _ = disable_raw_mode();
}

fn enter_raw_mode() -> io::Result<RawMode> {
    enable_raw_mode()?;
    Ok(RawMode {
        restore: restore_terminal,
    })
}

/// Read one keystroke from the terminal.
pub fn read_one_key() -> io::Result<PickKey> {
    let _raw_mode = enter_raw_mode()?;
    loop {
        if let Some(key) = keystroke_of(read()?) {
            return Ok(key);
        }
    }
}

/// The keystroke an event carries, when it carries one.
///
/// A resize, a focus change and a paste are events the popup waits through, and
/// so is the release of the key whose press it already answered.
fn keystroke_of(event: Event) -> Option<PickKey> {
    let Event::Key(key) = event else {
        return None;
    };
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let control_or_alt = KeyModifiers::CONTROL | KeyModifiers::ALT;
    Some(match key.code {
        KeyCode::Esc => PickKey::Escape,
        KeyCode::Enter => PickKey::Enter,
        KeyCode::Down => PickKey::Next,
        KeyCode::Up => PickKey::Previous,
        KeyCode::Char('c') if control => PickKey::Interrupt,
        KeyCode::Char('n') if control => PickKey::Next,
        KeyCode::Char('p') if control => PickKey::Previous,
        KeyCode::Char(character) if !key.modifiers.intersects(control_or_alt) => {
            PickKey::Character(character)
        }
        _ => PickKey::Other,
    })
}

#[cfg(test)]
mod tests;
