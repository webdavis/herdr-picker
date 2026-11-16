use super::PickerError;

pub(super) fn checked_sources(name: &str, sources: &[String]) -> Result<(), PickerError> {
    match sources.iter().find(|source| !is_lowercase_text(source)) {
        Some(source) => Err(PickerError::UnusableSource {
            name: name.to_string(),
            source: source.clone(),
        }),
        None => Ok(()),
    }
}

// A source is shown in the header and chooser and kept on one tab-separated line.
fn is_lowercase_text(source: &str) -> bool {
    !source.is_empty()
        && source
            .chars()
            .all(|character| !character.is_uppercase() && !character.is_control())
}

pub(super) fn is_bare_word(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}
