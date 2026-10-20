use herdr_picker_domain::PickerRow;

use super::history::{Message, detail, row};
use crate::builtin::paint::{blue, dim};

pub(super) fn link_rows(message: &Message) -> Vec<PickerRow> {
    links(&message.text)
        .map(|url| {
            let label = format!(
                "{}  {}",
                blue(url),
                dim(&format!("{}  {}", message.sender, message.date))
            );
            row(
                message,
                label,
                format!("URL: {url}\n{}", detail(message)),
                url,
            )
        })
        .collect()
}

// Each web address in the text, without the punctuation that ends a sentence around it.
fn links(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = text;
    std::iter::from_fn(move || {
        let start = [rest.find("http://"), rest.find("https://")]
            .into_iter()
            .flatten()
            .min()?;
        let found = &rest[start..];
        let end = found
            .find(|c: char| c.is_whitespace() || "<>\u{fffc}".contains(c))
            .unwrap_or(found.len());
        rest = &found[end..];
        Some(found[..end].trim_end_matches(['.', ',', ';', '!', '?', ')', ']', '}']))
    })
}

#[cfg(test)]
mod tests;
