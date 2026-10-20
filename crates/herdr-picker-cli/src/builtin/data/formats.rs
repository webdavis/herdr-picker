use std::path::Path;

// How a data file is written; each but SQLite is read into one JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Format {
    Json,
    Lines,
    Yaml,
    Toon,
    Sqlite,
}

impl Format {
    pub(super) fn of(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        Some(match extension.as_str() {
            "json" => Self::Json,
            "jsonl" | "ndjson" => Self::Lines,
            "yaml" | "yml" => Self::Yaml,
            "toon" => Self::Toon,
            "sqlite" | "sqlite3" | "db" => Self::Sqlite,
            _ => return None,
        })
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Lines => "json lines",
            Self::Yaml => "yaml",
            Self::Toon => "toon",
            Self::Sqlite => "sqlite",
        }
    }

    // The program that prints the file as JSON, for a format serde_json cannot read itself.
    pub(super) fn reader(self, path: &Path) -> Option<Vec<String>> {
        let path = path.to_string_lossy().into_owned();
        let argv: &[&str] = match self {
            Self::Yaml => &["yq", "-o", "json", "-I0"],
            Self::Toon => &["toon", "--decode"],
            Self::Json | Self::Lines | Self::Sqlite => return None,
        };
        Some(
            argv.iter()
                .map(|word| word.to_string())
                .chain([path])
                .collect(),
        )
    }
}

// A format a value is copied in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Output {
    Json,
    Yaml,
    Toon,
}

impl Output {
    pub(super) const ALL: [Self; 3] = [Self::Json, Self::Yaml, Self::Toon];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toon => "TOON",
        }
    }

    // The program that turns JSON on its input into this format.
    pub(super) fn converter(self) -> Option<&'static [&'static str]> {
        match self {
            Self::Json => None,
            Self::Yaml => Some(&["yq", "-p", "json", "-o", "yaml", "-"]),
            Self::Toon => Some(&["toon", "--encode"]),
        }
    }
}

#[cfg(test)]
mod tests;
