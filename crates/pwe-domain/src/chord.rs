use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    Ctrl(char),
    Alt(char),
    Plain(char),
}

impl Stroke {
    pub fn name(self) -> String {
        match self {
            Self::Ctrl(letter) => format!("ctrl-{letter}"),
            Self::Alt(letter) => format!("alt-{letter}"),
            Self::Plain(letter) => letter.to_string(),
        }
    }

    pub fn pick_key(self) -> Option<char> {
        match self {
            Self::Plain(key) => Some(key),
            Self::Ctrl(letter) => u8::try_from(letter)
                .ok()
                .filter(u8::is_ascii_alphabetic)
                .map(|letter| char::from(letter & CONTROL_BITS)),
            Self::Alt(_) => None,
        }
    }
}

const CONTROL_BITS: u8 = 0x1f;
const CARET_LETTER_OFFSET: u8 = 0x40;

pub fn pick_key_label(key: char) -> String {
    match u8::try_from(key) {
        Ok(byte) if byte.is_ascii_control() => {
            format!("^{}", char::from(byte | CARET_LETTER_OFFSET))
        }
        _ => key.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord(Vec<Stroke>);

impl Chord {
    pub fn parse(text: &str) -> Option<Self> {
        let strokes = text
            .split_whitespace()
            .map(stroke)
            .collect::<Option<Vec<_>>>()?;
        match strokes.first() {
            Some(Stroke::Ctrl(_) | Stroke::Alt(_)) => Some(Self(strokes)),
            _ => None,
        }
    }

    pub fn strokes(&self) -> &[Stroke] {
        &self.0
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(|stroke| stroke.name()).collect();
        formatter.write_str(&names.join(" "))
    }
}

fn stroke(word: &str) -> Option<Stroke> {
    if let Some(letter) = word.strip_prefix("ctrl-") {
        return only_letter(letter).map(|letter| Stroke::Ctrl(letter.to_ascii_lowercase()));
    }
    if let Some(letter) = word.strip_prefix("alt-") {
        return only_letter(letter).map(Stroke::Alt);
    }
    let mut characters = word.chars();
    match (characters.next(), characters.next()) {
        (Some(plain), None) if plain.is_ascii_alphabetic() || bindable_punctuation(plain) => {
            Some(Stroke::Plain(plain))
        }
        _ => None,
    }
}

fn bindable_punctuation(character: char) -> bool {
    character.is_ascii_punctuation() && !"'\"\\".contains(character)
}

fn only_letter(text: &str) -> Option<char> {
    let mut characters = text.chars();
    match (characters.next(), characters.next()) {
        (Some(letter), None) if letter.is_ascii_alphabetic() => Some(letter),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
