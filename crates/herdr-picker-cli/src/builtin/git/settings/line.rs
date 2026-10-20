use std::fs;

const SECRET_PARAMETERS: [&str; 5] = ["token", "access_token", "password", "api_key", "apikey"];

// The line a setting is written on in a Git config file, the last one when it is written more
// than once, since that is the one Git uses; 1 when it cannot be found.
pub(super) fn setting_line(path: &str, name: &str) -> u32 {
    let Some((section, key)) = name.rsplit_once('.') else {
        return 1;
    };
    let (head, sub) = section.split_once('.').unwrap_or((section, ""));
    let mut found = 1;
    let mut current: Option<(String, String)> = None;
    for (number, line) in fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .enumerate()
    {
        let mut rest = line.trim_start();
        if let Some(opened) = rest.strip_prefix('[') {
            let Some((inside, after)) = opened.split_once(']') else {
                continue;
            };
            current = Some(section_of(inside));
            rest = after.trim_start();
        }
        let entry: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        let tail = rest[entry.len()..].trim_start();
        let is_entry = !entry.is_empty() && (tail.is_empty() || tail.starts_with(['=', ';', '#']));
        let here = current
            .as_ref()
            .is_some_and(|(h, s)| h.eq_ignore_ascii_case(head) && s == sub);
        if is_entry && here && entry.eq_ignore_ascii_case(key) {
            found = number as u32 + 1;
        }
    }
    found
}

// [remote "origin"] and the older [remote.origin] name the same section.
fn section_of(inside: &str) -> (String, String) {
    let inside = inside.trim();
    if let Some((head, quoted)) = inside.split_once(char::is_whitespace) {
        return (
            head.to_string(),
            quoted.trim().trim_matches('"').to_string(),
        );
    }
    match inside.split_once('.') {
        Some((head, sub)) => (head.to_string(), sub.to_string()),
        None => (inside.to_string(), String::new()),
    }
}

// A user and password in a URL, and secret query parameters, never show.
pub(super) fn redact(text: &str) -> String {
    let mut shown = text.to_string();
    if let Some(scheme) = shown.find("://") {
        let start = scheme + 3;
        let end = shown[start..]
            .find(['/', ' '])
            .map_or(shown.len(), |end| start + end);
        if let Some(at) = shown[start..end].rfind('@') {
            shown.replace_range(start..start + at, "[redacted]");
        }
    }
    for parameter in SECRET_PARAMETERS {
        for separator in ['?', '&'] {
            let marker = format!("{separator}{parameter}=");
            if let Some(found) = shown.to_lowercase().find(&marker) {
                let start = found + marker.len();
                let end = shown[start..]
                    .find(['&', ' '])
                    .map_or(shown.len(), |end| start + end);
                shown.replace_range(start..end, "[redacted]");
            }
        }
    }
    shown
}
