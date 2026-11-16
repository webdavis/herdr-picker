use pwe_adapters::test_engine::RunFile;
use pwe_application::test_engine::{Clock, EventSink};
use pwe_domain::test_engine::TestEvent;
use pwe_protocol::test_engine::{EventLines, event_text};
use serde_json::{Value, json};

use super::output::{Format, encoded, print_out};
use super::system_clock::SystemClock;
use crate::builtin::clock::utc_time;

pub(super) struct EventPrinter {
    format: Format,
    lines: EventLines,
    collected: Vec<Value>,
    kept: Option<RunFile>,
}

impl EventPrinter {
    pub(super) fn new(format: Format, run_id: &str, kept: Option<RunFile>) -> Self {
        EventPrinter {
            format,
            lines: EventLines::new(run_id),
            collected: Vec::new(),
            kept,
        }
    }

    fn keep(&mut self, line: &str) {
        let Some(file) = &mut self.kept else {
            return;
        };
        if let Err(failure) = file.append(line) {
            eprintln!("pwe: this run is no longer kept in the test history: {failure}");
            self.kept = None;
        }
    }

    pub(super) fn finish(self) {
        if matches!(self.format, Format::Json | Format::Toon) {
            print_out(&encoded(
                self.format,
                &json!({ "events": self.collected }),
                String::new,
            ));
        }
    }
}

impl EventSink for EventPrinter {
    fn emit(&mut self, event: &TestEvent) {
        let line = self.lines.line(&utc_time(SystemClock.now_ms()), event);
        self.keep(&line);
        match self.format {
            Format::Jsonl => print_out(&format!("{line}\n")),
            Format::Text => print_out(&event_text(event)),
            Format::Json | Format::Toon => self
                .collected
                .push(serde_json::from_str(&line).unwrap_or(Value::Null)),
        }
    }
}
