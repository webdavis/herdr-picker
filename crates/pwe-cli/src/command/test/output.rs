use std::io::{self, IsTerminal, Write};

use pwe_protocol::test_engine::{Hint, TestError, TestErrorCode};
use serde_json::Value;

use crate::command::CommandError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Format {
    Json,
    Jsonl,
    Toon,
    Text,
}

const FORMATS: [&str; 4] = ["json", "jsonl", "toon", "text"];
pub(super) const COULD_NOT_START: u8 = 2;

pub(super) fn chosen_format(flag: Option<&str>) -> Result<Format, TestError> {
    match flag {
        None if io::stdout().is_terminal() => Ok(Format::Text),
        None | Some("jsonl") => Ok(Format::Jsonl),
        Some("json") => Ok(Format::Json),
        Some("toon") => Ok(Format::Toon),
        Some("text") => Ok(Format::Text),
        Some(other) => Err(TestError::new(
            TestErrorCode::BadFlag,
            format!("pwe test has no format `{other}`."),
        )
        .with_hint(Hint {
            accepted: FORMATS.map(String::from).to_vec(),
            ..Hint::default()
        })),
    }
}

pub(super) fn encoded(format: Format, value: &Value, text: impl FnOnce() -> String) -> String {
    match format {
        Format::Json | Format::Jsonl => format!("{value}\n"),
        Format::Toon => toon_format::encode_default(value)
            .map(|toon| format!("{toon}\n"))
            .unwrap_or_else(|_| format!("{value}\n")),
        Format::Text => text(),
    }
}

pub(super) fn print_out(text: &str) {
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
}

pub(super) fn exit_with(code: i32) -> Result<(), CommandError> {
    match u8::try_from(code) {
        Ok(0) => Ok(()),
        Ok(code) => Err(CommandError::Exit(code)),
        Err(_) => Err(CommandError::Exit(1)),
    }
}

pub(super) fn refused(format: Option<Format>, error: &TestError) -> CommandError {
    let format = format.unwrap_or_else(|| chosen_format(None).unwrap_or(Format::Jsonl));
    let text = encoded(format, &error.json(), || error.text());
    let _ = io::stderr().write_all(text.as_bytes());
    CommandError::Exit(COULD_NOT_START)
}
