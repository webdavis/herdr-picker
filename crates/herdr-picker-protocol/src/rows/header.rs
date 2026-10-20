use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireHeader {
    header: String,
}

// A line a picker prints to say what its step is about; the header shows it under the title.
pub fn parse_header(line: &str) -> Option<String> {
    serde_json::from_str::<WireHeader>(line)
        .ok()
        .map(|wire| wire.header)
}

pub fn encode_header(text: &str) -> String {
    serde_json::to_string(&WireHeader {
        header: text.to_string(),
    })
    .expect("a header always encodes")
}
