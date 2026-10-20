use serde_json::{Value, json};

pub(super) use super::decode::Decoding;

// One step from a value to a value inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Segment {
    Key(String),
    Index(usize),
    // The string here, read as what it holds.
    Decoded(Decoding),
}

// Where a value sits in a document: the segments from the top.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Place(pub(super) Vec<Segment>);

impl Place {
    pub(super) fn child(&self, segment: Segment) -> Self {
        let mut segments = self.0.clone();
        segments.push(segment);
        Self(segments)
    }

    pub(super) fn label(&self) -> String {
        self.render(false, false)
    }

    // The jq filter that reaches the value, decodings included.
    pub(super) fn jq(&self) -> String {
        self.render(true, false)
    }

    // The path with every array item at once, as a label and as the jq filter.
    pub(super) fn shape(&self) -> String {
        self.render(false, true)
    }

    pub(super) fn shape_jq(&self) -> String {
        self.render(true, true)
    }

    pub(super) fn decoded(&self) -> bool {
        self.0
            .iter()
            .any(|segment| matches!(segment, Segment::Decoded(_)))
    }

    pub(super) fn last_key(&self) -> Option<&str> {
        self.0.iter().rev().find_map(|segment| match segment {
            Segment::Key(key) => Some(key.as_str()),
            _ => None,
        })
    }

    pub(super) fn raw(&self) -> Vec<Value> {
        self.0
            .iter()
            .map_while(|segment| match segment {
                Segment::Key(key) => Some(json!(key)),
                Segment::Index(index) => Some(json!(index)),
                Segment::Decoded(_) => None,
            })
            .collect()
    }

    pub(super) fn state(&self) -> String {
        let segments: Vec<Value> = self
            .0
            .iter()
            .map(|segment| match segment {
                Segment::Key(key) => json!(key),
                Segment::Index(index) => json!(index),
                Segment::Decoded(decoding) => json!({"decoded": decoding.name()}),
            })
            .collect();
        Value::Array(segments).to_string()
    }

    pub(super) fn parse(state: &str) -> Option<Self> {
        let segments: Vec<Value> = serde_json::from_str(state).ok()?;
        segments
            .iter()
            .map(|segment| match segment {
                Value::String(key) => Some(Segment::Key(key.clone())),
                Value::Number(index) => Some(Segment::Index(index.as_u64()? as usize)),
                _ => match segment["decoded"].as_str()? {
                    "base64" => Some(Segment::Decoded(Decoding::Base64)),
                    "json" => Some(Segment::Decoded(Decoding::Json)),
                    _ => None,
                },
            })
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    fn render(&self, pipes: bool, every_index: bool) -> String {
        let mut parts = vec![String::new()];
        for segment in &self.0 {
            let part = parts.last_mut().expect("parts start with one");
            match segment {
                Segment::Key(key) => part.push_str(&key_step(key)),
                Segment::Index(index) => {
                    if part.is_empty() {
                        part.push('.');
                    }
                    match every_index {
                        true => part.push_str("[]"),
                        false => part.push_str(&format!("[{index}]")),
                    }
                }
                Segment::Decoded(decoding) if pipes => {
                    parts.push(decoding.filter().to_string());
                    parts.push(String::new());
                }
                Segment::Decoded(_) => {}
            }
        }
        let parts: Vec<String> = parts.into_iter().filter(|part| !part.is_empty()).collect();
        match parts.is_empty() {
            true => ".".to_string(),
            false => parts.join(" | "),
        }
    }
}

fn key_step(key: &str) -> String {
    let plain = key.starts_with(|first: char| first.is_ascii_alphabetic() || first == '_')
        && key
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '_');
    match plain {
        true => format!(".{key}"),
        false => format!(".{}", json!(key)),
    }
}

#[cfg(test)]
mod tests;
