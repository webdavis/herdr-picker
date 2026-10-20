#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    Ctrl(char),
    Alt(char),
    Plain(char),
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

fn stroke(word: &str) -> Option<Stroke> {
    if let Some(letter) = word.strip_prefix("ctrl-") {
        return only_letter(letter).map(|letter| Stroke::Ctrl(letter.to_ascii_lowercase()));
    }
    if let Some(letter) = word.strip_prefix("alt-") {
        return only_letter(letter).map(Stroke::Alt);
    }
    only_letter(word).map(Stroke::Plain)
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
