use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Deserializer, Value};

use super::formats::Format;
use crate::builtin::program::{ran, run};

const CLIPBOARD: &str = "clipboard";
const READ_DEADLINE: Duration = Duration::from_secs(20);

// Where a document's text comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Origin {
    File(PathBuf),
    // Read with pbpaste, never written.
    Clipboard,
}

impl Origin {
    pub(super) fn parse(argument: &str) -> Self {
        match argument {
            CLIPBOARD => Self::Clipboard,
            path => Self::File(PathBuf::from(path)),
        }
    }

    pub(super) fn argument(&self) -> String {
        match self {
            Self::File(path) => path.to_string_lossy().into_owned(),
            Self::Clipboard => CLIPBOARD.to_string(),
        }
    }

    pub(super) fn format(&self) -> Option<Format> {
        match self {
            Self::File(path) => Format::of(path),
            Self::Clipboard => None,
        }
    }

    pub(super) fn read(&self) -> Result<Value, String> {
        let path = match self {
            Self::Clipboard => return clipboard(),
            Self::File(path) => path,
        };
        let format = self.format();
        let text = match format.and_then(|format| format.reader(path)) {
            Some(argv) => run(
                &argv.iter().map(String::as_str).collect::<Vec<_>>(),
                READ_DEADLINE,
            )?,
            None => fs::read_to_string(path)
                .map_err(|failure| format!("{}: {failure}", path.display()))?,
        };
        match format {
            Some(Format::Toon) => {
                toon_format::decode_default(&text).map_err(|failure| format!("not TOON: {failure}"))
            }
            _ => values(&text),
        }
    }
}

// One value, or each of a stream of values (JSON Lines, YAML documents) as an array.
pub(super) fn values(text: &str) -> Result<Value, String> {
    let mut read = Deserializer::from_str(text)
        .into_iter::<Value>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|failure| format!("not JSON: {failure}"))?;
    match read.len() {
        1 => Ok(read.remove(0)),
        _ => Ok(Value::Array(read)),
    }
}

fn clipboard() -> Result<Value, String> {
    let text = run(&["pbpaste"], READ_DEADLINE)?;
    values(&text).or_else(|_| {
        let yaml = ran(
            &["yq", "-p", "yaml", "-o", "json", "-I0", "-"],
            Some(&text),
            READ_DEADLINE,
        )?;
        match values(&yaml.stdout) {
            Ok(value) if yaml.succeeded && super::tree::container(&value) => Ok(value),
            _ => Err("The clipboard holds no JSON or YAML.".to_string()),
        }
    })
}
