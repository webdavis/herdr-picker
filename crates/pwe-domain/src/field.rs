#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    Insert(char),
    Backward,
    Forward,
    WordBackward,
    WordForward,
    Start,
    End,
    DeleteBackward,
    DeleteForward,
    CutToStart,
    CutToEnd,
    CutWordBackward,
    CutWordForward,
    Paste,
}

pub trait TextField {
    fn apply(&mut self, edit: Edit);
    fn text(&self) -> String;
    fn cursor(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.text().is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadlineField {
    text: Vec<char>,
    cursor: usize,
    cut: Vec<char>,
}

impl TextField for ReadlineField {
    fn apply(&mut self, edit: Edit) {
        let end = self.text.len();
        match edit {
            Edit::Insert(character) => {
                self.text.insert(self.cursor, character);
                self.cursor += 1;
            }
            Edit::Backward => self.cursor = self.cursor.saturating_sub(1),
            Edit::Forward => self.cursor = (self.cursor + 1).min(end),
            Edit::WordBackward => self.cursor = self.word_start(),
            Edit::WordForward => self.cursor = self.word_end(),
            Edit::Start => self.cursor = 0,
            Edit::End => self.cursor = end,
            Edit::DeleteBackward if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.cursor);
            }
            Edit::DeleteForward if self.cursor < end => {
                self.text.remove(self.cursor);
            }
            Edit::DeleteBackward | Edit::DeleteForward => {}
            Edit::CutToStart => self.cut(0, self.cursor),
            Edit::CutToEnd => self.cut(self.cursor, end),
            Edit::CutWordBackward => self.cut(self.word_start(), self.cursor),
            Edit::CutWordForward => self.cut(self.cursor, self.word_end()),
            Edit::Paste => {
                let pasted = self.cut.clone();
                let count = pasted.len();
                self.text.splice(self.cursor..self.cursor, pasted);
                self.cursor += count;
            }
        }
    }

    fn text(&self) -> String {
        self.text.iter().collect()
    }

    fn cursor(&self) -> usize {
        self.cursor
    }
}

impl ReadlineField {
    fn cut(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        self.cut = self.text.drain(from..to).collect();
        self.cursor = from;
    }

    fn word_start(&self) -> usize {
        let mut position = self.cursor;
        while position > 0 && !is_word(self.text[position - 1]) {
            position -= 1;
        }
        while position > 0 && is_word(self.text[position - 1]) {
            position -= 1;
        }
        position
    }

    fn word_end(&self) -> usize {
        let end = self.text.len();
        let mut position = self.cursor;
        while position < end && !is_word(self.text[position]) {
            position += 1;
        }
        while position < end && is_word(self.text[position]) {
            position += 1;
        }
        position
    }
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric()
}

#[cfg(test)]
mod tests;
