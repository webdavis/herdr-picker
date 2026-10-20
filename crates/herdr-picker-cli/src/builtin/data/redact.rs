use serde_json::Value;

pub(super) const HIDDEN: &str = "[hidden]";

// Key names that hold secrets, compared lowercased with separators dropped.
const SECRET_WORDS: [&str; 10] = [
    "password",
    "passwd",
    "secret",
    "token",
    "apikey",
    "accesskey",
    "privatekey",
    "authorization",
    "cookie",
    "phone",
];
const SECRET_KEYS: [&str; 1] = ["mobile"];

// The value with its secrets hidden: every string under a secret key, and phone numbers anywhere.
// Only strings: counts such as input_tokens stay.
pub(super) fn redacted(value: &Value, key: Option<&str>) -> Value {
    hidden(value, key.is_some_and(secret))
}

fn hidden(value: &Value, under_secret: bool) -> Value {
    match value {
        Value::String(text) if under_secret || phone(text) => Value::String(HIDDEN.to_string()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, field)| (key.clone(), hidden(field, under_secret || secret(key))))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| hidden(item, under_secret))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn secret(key: &str) -> bool {
    let plain: String = key
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase();
    SECRET_WORDS.iter().any(|word| plain.contains(word)) || SECRET_KEYS.contains(&plain.as_str())
}

// Ten to fifteen digits written the way people write phone numbers: with a + or separators.
fn phone(text: &str) -> bool {
    let text = text.trim();
    let digits = text.chars().filter(char::is_ascii_digit).count();
    let written = text
        .chars()
        .all(|letter| letter.is_ascii_digit() || " +-().".contains(letter));
    let separated = text.starts_with('+') || text.contains([' ', '-', '(', '.']);
    written && separated && (10..=15).contains(&digits)
}

#[cfg(test)]
mod tests;
