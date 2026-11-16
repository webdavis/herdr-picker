use std::path::Path;

use super::words;

mod lines;

use lines::{escaped, go_test, swift_test};

// A test framework that lists its tests when asked and runs one of them by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Framework {
    Swift,
    Rust,
    Pytest,
    Go,
}

impl Framework {
    pub(super) const ALL: [Self; 4] = [Self::Swift, Self::Rust, Self::Pytest, Self::Go];

    // The name a saved listing is filed under.
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Swift => "swift",
            Self::Rust => "rust",
            Self::Pytest => "pytest",
            Self::Go => "go",
        }
    }

    pub(super) fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|framework| framework.name() == name)
    }

    // The files that mark a project this framework tests.
    pub(super) fn markers(self) -> &'static [&'static str] {
        match self {
            Self::Swift => &["Package.swift"],
            Self::Rust => &["Cargo.toml"],
            Self::Pytest => &["pytest.ini", "pyproject.toml", "setup.cfg"],
            Self::Go => &["go.mod"],
        }
    }

    pub(super) fn discover_label(self) -> &'static str {
        match self {
            Self::Swift => "Discover Swift tests (XCTest and Swift Testing)",
            Self::Rust => "Discover Rust tests",
            Self::Pytest => "Discover pytest tests",
            Self::Go => "Discover Go tests",
        }
    }

    // The command that prints the tests of the project in this folder.
    pub(super) fn listing(self, folder: &Path) -> String {
        let folder = folder.to_string_lossy();
        match self {
            Self::Swift => words(&["swift", "test", "--package-path", &folder, "list"]),
            Self::Rust => format!(
                "{} -- --list --format terse",
                words(&["cargo", "test", "--manifest-path", &manifest(&folder)])
            ),
            Self::Pytest => "python3 -m pytest --collect-only -q".to_string(),
            Self::Go => "go test ./... -list .".to_string(),
        }
    }

    // The label and command of the row that runs every test, for frameworks with one.
    pub(super) fn everything(self, folder: &Path) -> Option<(&'static str, String)> {
        let folder = folder.to_string_lossy();
        match self {
            Self::Swift => Some((
                "Run all Swift tests",
                words(&["swift", "test", "--package-path", &folder]),
            )),
            Self::Rust => Some((
                "Run all Rust tests",
                words(&["cargo", "test", "--manifest-path", &manifest(&folder)]),
            )),
            Self::Pytest | Self::Go => None,
        }
    }

    // The tests a listing printed, each once, in the order printed.
    pub(super) fn tests(self, printed: &str) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        for line in printed.lines() {
            let test = match self {
                Self::Swift => Some(line.trim()).filter(|line| swift_test(line)),
                Self::Rust => line.strip_suffix(": test"),
                Self::Pytest => {
                    (line.contains(".py::") && !line.starts_with([' ', '<'])).then_some(line.trim())
                }
                Self::Go => Some(line).filter(|line| go_test(line)),
            };
            if let Some(test) = test.filter(|test| !found.iter().any(|seen| seen == test)) {
                found.push(test.to_string());
            }
        }
        found
    }

    // The command that runs only this test of the project in the folder.
    pub(super) fn one(self, folder: &Path, test: &str) -> String {
        let shown = folder.to_string_lossy();
        let exactly = format!("^{}$", escaped(test));
        match self {
            Self::Swift => words(&[
                "swift",
                "test",
                "--package-path",
                &shown,
                "--filter",
                &exactly,
            ]),
            Self::Rust => format!(
                "{} -- --exact",
                words(&["cargo", "test", "--manifest-path", &manifest(&shown), test])
            ),
            Self::Pytest => words(&[
                "python3",
                "-m",
                "pytest",
                &folder.join(test).to_string_lossy(),
            ]),
            Self::Go => words(&["go", "-C", &shown, "test", "./...", "-run", &exactly]),
        }
    }
}

fn manifest(folder: &str) -> String {
    format!("{folder}/Cargo.toml")
}
