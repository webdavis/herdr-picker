use super::PickerError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Engine {
    #[default]
    Fzf,
    Pwe,
}

const ENGINE_WORDS: [(&str, Engine); 2] = [("fzf", Engine::Fzf), ("pwe", Engine::Pwe)];

impl Engine {
    pub fn named(picker: &str, word: &str) -> Result<Self, PickerError> {
        ENGINE_WORDS
            .iter()
            .find(|(accepted, _)| *accepted == word)
            .map(|(_, engine)| *engine)
            .ok_or_else(|| PickerError::UnknownEngine {
                name: picker.to_string(),
                engine: word.to_string(),
            })
    }
}
