use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

use super::formats::Format;
use super::load::Origin;
use super::place::Place;
use crate::builtin::program::run;

const DEADLINE: Duration = Duration::from_secs(20);

pub(super) enum Lines {
    ByPath(HashMap<String, u32>),
    ByRecord,
    Unknown,
}

impl Lines {
    pub(super) fn of(&self, place: &Place) -> Option<u32> {
        let mut raw = place.raw();
        match self {
            Self::ByRecord => Some(raw.first()?.as_u64()? as u32 + 1),
            Self::ByPath(lines) => loop {
                if let Some(line) = lines.get(&Value::Array(raw.clone()).to_string()) {
                    return Some(*line);
                }
                raw.pop()?;
            },
            Self::Unknown => None,
        }
    }
}

pub(super) fn lines(origin: &Origin) -> Lines {
    match (origin, origin.format()) {
        (_, Some(Format::Lines)) => Lines::ByRecord,
        (Origin::File(path), Some(Format::Json | Format::Yaml)) => {
            let path = path.to_string_lossy();
            let argv = [
                "yq",
                "-p",
                "yaml",
                "-o",
                "json",
                "-I0",
                "[.. | [path, line]]",
                &path,
            ];
            let listed = run(&argv, DEADLINE).unwrap_or_default();
            let pairs: Vec<(Value, u32)> = serde_json::from_str(&listed).unwrap_or_default();
            Lines::ByPath(
                pairs
                    .into_iter()
                    .map(|(path, line)| (path.to_string(), line))
                    .collect(),
            )
        }
        _ => Lines::Unknown,
    }
}
