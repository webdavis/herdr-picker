#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntrustedText {
    pub text: String,
    pub stripped: usize,
}

impl UntrustedText {
    pub fn new(raw: &str) -> Self {
        let text: String = raw
            .chars()
            .filter(|character| is_kept(*character))
            .collect();
        UntrustedText {
            stripped: raw.chars().count() - text.chars().count(),
            text,
        }
    }
}

pub fn printed_name(raw: &str) -> String {
    raw.chars()
        .filter(|character| !character.is_control())
        .collect()
}

fn is_kept(character: char) -> bool {
    !character.is_control() || character == '\n' || character == '\t'
}

#[cfg(test)]
mod tests;
