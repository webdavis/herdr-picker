use std::io;

use pwe_domain::PickerRow;

use super::deliver;
use crate::picker_run::row_sink::RowSink;

#[derive(Default)]
struct RecordedSink {
    taken: Vec<(usize, String, String)>,
    headers: Vec<String>,
}

impl RowSink for RecordedSink {
    fn take(&mut self, number: usize, row: PickerRow, line: &str) -> io::Result<()> {
        self.taken.push((number, row.label, line.to_string()));
        Ok(())
    }

    fn header(&mut self, header: &str) {
        self.headers.push(header.to_string());
    }

    fn finish(self) -> Vec<PickerRow> {
        Vec::new()
    }
}

const PRINTED: &str = r#"{"header":"Pick one"}
{"label":"beta","enter":"none"}
{"label":"alpha","enter":"none"}
{"label":"gamma","enter":"none"}
"#;

#[test]
fn rows_reach_the_sink_in_the_order_printed() {
    let mut sink = RecordedSink::default();

    let bad_row = deliver(PRINTED.as_bytes(), &mut sink);

    assert_eq!(bad_row, None);
    assert_eq!(sink.headers, ["Pick one"]);
    assert_eq!(
        sink.taken,
        [
            (
                0,
                "beta".to_string(),
                r#"{"label":"beta","enter":"none"}"#.to_string()
            ),
            (
                1,
                "alpha".to_string(),
                r#"{"label":"alpha","enter":"none"}"#.to_string()
            ),
            (
                2,
                "gamma".to_string(),
                r#"{"label":"gamma","enter":"none"}"#.to_string()
            ),
        ]
    );
}
