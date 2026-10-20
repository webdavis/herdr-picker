use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Decoding {
    Base64,
    Json,
}

impl Decoding {
    pub(super) fn filter(self) -> &'static str {
        match self {
            Self::Base64 => "@base64d",
            Self::Json => "fromjson",
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Base64 => "base64",
            Self::Json => "json",
        }
    }
}

// Shorter runs are too often plain words that happen to fit the alphabet.
const SHORTEST_BASE64: usize = 16;

// What a string holds when it is JSON or base64 text, with the decodings that reach it.
pub(super) fn decoded(text: &str) -> Option<(Vec<Decoding>, Value)> {
    let trimmed = text.trim();
    if trimmed.starts_with(['{', '[']) {
        let held: Value = serde_json::from_str(trimmed).ok()?;
        return Some((vec![Decoding::Json], held));
    }
    let printable = |text: &String| {
        !text.trim().is_empty()
            && text
                .chars()
                .all(|letter| !letter.is_control() || matches!(letter, '\n' | '\r' | '\t'))
    };
    let base64_shaped = trimmed.len() >= SHORTEST_BASE64 && trimmed.len().is_multiple_of(4);
    let text = base64_shaped
        .then(|| base64(trimmed))
        .flatten()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .filter(printable)?;
    let mut decodings = vec![Decoding::Base64];
    match decoded(&text) {
        Some((further, held)) => {
            decodings.extend(further);
            Some((decodings, held))
        }
        None => Some((decodings, Value::String(text))),
    }
}

pub(super) fn base64(text: &str) -> Option<Vec<u8>> {
    let digits = text.trim_end_matches('=');
    if digits.len() % 4 == 1 || text.len() - digits.len() > 2 {
        return None;
    }
    let mut bytes = Vec::with_capacity(digits.len() * 3 / 4);
    let (mut buffer, mut bits) = (0u32, 0);
    for letter in digits.bytes() {
        buffer = buffer << 6 | u32::from(sextet(letter)?);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }
    Some(bytes)
}

fn sextet(letter: u8) -> Option<u8> {
    match letter {
        b'A'..=b'Z' => Some(letter - b'A'),
        b'a'..=b'z' => Some(letter - b'a' + 26),
        b'0'..=b'9' => Some(letter - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
