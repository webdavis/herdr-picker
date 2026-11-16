use std::sync::mpsc;
use std::time::{Duration, Instant};

use pwe_domain::finder::Input;
use pwe_domain::{Action, PickerRow};

use super::Arrivals;

fn row(label: &str) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Nothing,
        keys: Vec::new(),
        group: None,
    }
}

fn arrived(labels: &[&str]) -> Input {
    Input::RowsArrived(labels.iter().map(|label| row(label)).collect())
}

#[test]
fn rows_arriving_within_fifty_milliseconds_of_a_ranking_wait_for_the_next() {
    let (inputs, received) = mpsc::channel();
    let mut arrivals = Arrivals::new(received);
    let start = Instant::now();
    inputs.send(arrived(&["one"])).expect("sent");
    inputs.send(arrived(&["two"])).expect("sent");

    let first = arrivals.ready(start);
    inputs.send(arrived(&["three"])).expect("sent");
    let too_soon = arrivals.ready(start + Duration::from_millis(30));
    let later = arrivals.ready(start + Duration::from_millis(50));

    assert_eq!(first, [arrived(&["one", "two"])]);
    assert_eq!(too_soon, []);
    assert_eq!(later, [arrived(&["three"])]);
}

#[test]
fn waiting_rows_come_before_the_end_of_the_rows() {
    let (inputs, received) = mpsc::channel();
    let mut arrivals = Arrivals::new(received);
    let start = Instant::now();
    inputs.send(arrived(&["one"])).expect("sent");
    let _ = arrivals.ready(start);
    inputs.send(arrived(&["two"])).expect("sent");
    inputs.send(Input::RowsEnded).expect("sent");

    assert_eq!(
        arrivals.ready(start + Duration::from_millis(1)),
        [arrived(&["two"]), Input::RowsEnded]
    );
}

#[test]
fn a_preview_does_not_wait_for_the_rows() {
    let (inputs, received) = mpsc::channel();
    let mut arrivals = Arrivals::new(received);
    let start = Instant::now();
    inputs.send(arrived(&["zero"])).expect("sent");
    let _ = arrivals.ready(start);
    inputs.send(arrived(&["one"])).expect("sent");
    let preview = Input::PreviewArrived {
        row: 0,
        text: "text".to_string(),
    };
    inputs.send(preview.clone()).expect("sent");

    assert_eq!(arrivals.ready(start), [preview]);
}
