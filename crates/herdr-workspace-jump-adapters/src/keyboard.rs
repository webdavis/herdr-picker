use std::io;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use herdr_workspace_jump_application::InputSource;
use herdr_workspace_jump_domain::PickKey;

pub struct TerminalInput;

impl InputSource for TerminalInput {
    fn next_input(&mut self) -> io::Result<PickKey> {
        let _raw_mode = enter_raw_mode()?;
        loop {
            if let Some(key) = keystroke_of(read()?) {
                return Ok(key);
            }
        }
    }
}

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

fn keystroke_of(event: Event) -> Option<PickKey> {
    let Event::Key(key) = event else {
        return None;
    };
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    Some(match key.code {
        KeyCode::Esc => PickKey::Escape,
        KeyCode::Enter => PickKey::Enter,
        KeyCode::Backspace if control || alt => PickKey::WordBackspace,
        KeyCode::Backspace => PickKey::Backspace,
        KeyCode::Delete => PickKey::Delete,
        KeyCode::Up => PickKey::Up,
        KeyCode::Down => PickKey::Down,
        KeyCode::Left => PickKey::Left,
        KeyCode::Right => PickKey::Right,
        KeyCode::Home => PickKey::Home,
        KeyCode::End => PickKey::End,
        KeyCode::Char(_) if control && alt => PickKey::Other,
        KeyCode::Char(character) if control => PickKey::Control(character.to_ascii_lowercase()),
        KeyCode::Char(character) if alt => PickKey::Alt(character),
        KeyCode::Char(character) => PickKey::Character(character),
        _ => PickKey::Other,
    })
}

#[cfg(test)]
mod tests;
