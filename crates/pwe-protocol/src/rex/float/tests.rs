use serde_json::{Value, json};

use super::*;
use crate::rex::parse_key_facts;

const WHOLE_WINDOW: &str = include_str!("../fixtures/sleep-in-a-whole-window.json");
const RIGHT_HALF: &str = include_str!("../fixtures/sleep-in-the-right-half.json");
const IN_A_FLOAT: &str = include_str!("../fixtures/sleep-in-a-float.json");

fn requested(facts: &str) -> Value {
    let facts = parse_key_facts(facts).expect("facts");
    serde_json::from_str(&float_request(&facts, &["pwe".to_string()])).expect("JSON")
}

fn bounds(request: &Value) -> Vec<f64> {
    request["bounds"]
        .as_array()
        .expect("bounds")
        .iter()
        .map(|side| (side.as_f64().expect("a number") * 1000.0).round() / 1000.0)
        .collect()
}

#[test]
fn the_float_is_centred_and_starts_at_the_picker_size() {
    let request = requested(WHOLE_WINDOW);

    assert_eq!(
        request["layout"]["block"]["options"]["size"],
        json!({"columns": 64, "rows": 19})
    );
    assert_eq!(bounds(&request), vec![0.1, 0.104, 0.8, 0.792]);
}

#[test]
fn a_pane_in_half_the_window_sizes_the_float_to_the_whole_window() {
    let request = requested(RIGHT_HALF);

    assert_eq!(
        request["layout"]["block"]["options"]["size"],
        json!({"columns": 128, "rows": 19})
    );
    assert_eq!(bounds(&request), vec![0.1, 0.104, 0.8, 0.792]);
}

#[test]
fn a_pane_that_is_itself_a_float_counts_its_layers_share_of_the_window() {
    let request = requested(IN_A_FLOAT);

    assert_eq!(
        request["layout"]["block"]["options"]["size"],
        json!({"columns": 96, "rows": 24})
    );
    assert_eq!(bounds(&request), vec![0.1, 0.1, 0.8, 0.8]);
}

#[test]
fn the_float_runs_through_the_login_environment_so_pickers_find_their_tools() {
    let request = requested(WHOLE_WINDOW);

    let options = request["layout"]["block"]["options"]
        .as_object()
        .expect("options");
    assert!(!options.contains_key("shell"), "{options:?}");
}
