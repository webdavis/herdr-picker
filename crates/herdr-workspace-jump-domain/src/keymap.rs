use crate::field::Edit;
use crate::pick::{Command, Mode};

pub(crate) const SEARCH_CHARACTER: char = '/';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKey {
    Character(char),
    Control(char),
    Escape,
    Enter,
    Backspace,
    Up,
    Down,
    Other,
}

pub fn command(key: PickKey, mode: Mode) -> Command {
    use PickKey::{Backspace, Character, Control, Down, Enter, Escape, Up};
    let searching = mode == Mode::Searching;
    match key {
        Escape | Control('c') => Command::Cancel,
        Enter => Command::Accept,
        Down | Control('n') => Command::Next,
        Up | Control('p') => Command::Previous,
        Character(SEARCH_CHARACTER) if !searching => Command::Search,
        Character(character) if !searching => Command::Key(character),
        _ if !searching => Command::Unbound,
        Character(character) => Command::Edit(Edit::Insert(character)),
        Backspace => Command::Erase,
        _ => Command::Unbound,
    }
}

#[cfg(test)]
mod tests;
