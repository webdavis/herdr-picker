use super::*;

const TWO_WINDOWS: &str = r#"{"active_window_id":"window:two","windows":[
    {"window_id":"window:one","focused_block_id":"block:one","layers":[]},
    {"window_id":"window:two","focused_block_id":"block:two","layers":[{"bounds":[0,0,1,1],"layout":{"rect":[0,0.5,1,0.5],"block_id":"block:two"}}]}
]}"#;

#[test]
fn the_focused_block_is_the_active_windows() {
    assert_eq!(
        focused_block_in_view(TWO_WINDOWS),
        Ok("block:two".to_string())
    );
}

#[test]
fn a_view_with_no_active_window_names_no_block() {
    assert!(focused_block_in_view(r#"{"windows":[]}"#).is_err());
}

#[test]
fn the_window_is_the_focused_blocks_size_over_its_share() {
    let window = window_in_view(TWO_WINDOWS, "block:two", r#"{"columns":120,"rows":20}"#);

    assert_eq!(
        window,
        Ok(Size {
            columns: 120,
            rows: 40
        })
    );
}

#[test]
fn a_new_layer_answer_names_its_layer_and_block() {
    let opened = opened_layer(r#"{"layer_id":"layer:one","block_ids":["block:one"],"revision":1}"#);

    assert_eq!(
        opened,
        Ok(OpenedLayer {
            layer: "layer:one".to_string(),
            block: "block:one".to_string()
        })
    );
    assert!(opened_layer(r#"{"layer_id":"layer:one","block_ids":[]}"#).is_err());
}
