use crate::field::Edit;
use crate::pick::{Command, Mode};

pub(crate) const SEARCH_CHARACTER: char = '/';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKey {
    Character(char),
    Control(char),
    Alt(char),
    Escape,
    Enter,
    Backspace,
    WordBackspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Other,
}

pub trait Keymap {
    fn command(&self, key: PickKey, mode: Mode) -> Command;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Legend {
    pub search: &'static str,
    pub keys: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReadlineKeymap;

impl ReadlineKeymap {
    pub const LEGEND: Legend = Legend {
        search: "/ search",
        keys: "ctrl+n/p move  enter jump  esc close",
    };
}

impl Keymap for ReadlineKeymap {
    fn command(&self, key: PickKey, mode: Mode) -> Command {
        use PickKey::{
            Alt, Backspace, Character, Control, Delete, Down, End, Enter, Escape, Home, Left,
            Right, Up, WordBackspace,
        };
        let searching = mode == Mode::Searching;
        match key {
            Escape | Control('c' | 'g') => Command::Cancel,
            Enter => Command::Accept,
            Down | Control('n') => Command::Next,
            Up | Control('p') => Command::Previous,
            Character(SEARCH_CHARACTER) if !searching => Command::Search,
            Character(character) if !searching => Command::Key(character),
            _ if !searching => Command::Unbound,
            Character(character) => Command::Edit(Edit::Insert(character)),
            Backspace | Control('h') => Command::Erase,
            Delete | Control('d') => Command::Edit(Edit::DeleteForward),
            Left | Control('b') => Command::Edit(Edit::Backward),
            Right | Control('f') => Command::Edit(Edit::Forward),
            Home | Control('a') => Command::Edit(Edit::Start),
            End | Control('e') => Command::Edit(Edit::End),
            Alt('b') => Command::Edit(Edit::WordBackward),
            Alt('f') => Command::Edit(Edit::WordForward),
            Control('u') => Command::Edit(Edit::CutToStart),
            Control('k') => Command::Edit(Edit::CutToEnd),
            Control('w') | WordBackspace => Command::Edit(Edit::CutWordBackward),
            Alt('d') => Command::Edit(Edit::CutWordForward),
            Control('y') => Command::Edit(Edit::Paste),
            _ => Command::Unbound,
        }
    }
}

#[cfg(test)]
mod tests;
