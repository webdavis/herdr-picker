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
}

impl EventPrinter {
    pub(super) fn new(format: Format, run_id: &str) -> Self {
        EventPrinter {
            format,
            lines: EventLines::new(run_id),
            collected: Vec::new(),
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
        match self.format {
            Format::Jsonl => print_out(&format!("{line}\n")),
            Format::Text => print_out(&event_text(event)),
            Format::Json | Format::Toon => self
                .collected
                .push(serde_json::from_str(&line).unwrap_or(Value::Null)),
        }
    }
}
