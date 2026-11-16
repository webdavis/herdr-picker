#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RowLook {
    pub leaders: bool,
    pub stripes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    CatppuccinMocha,
}

pub fn unstyled(text: &str) -> String {
    let mut plain = String::new();
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character == '\x1b' {
            characters.by_ref().find(char::is_ascii_alphabetic);
        } else {
            plain.push(character);
        }
    }
    plain
}
