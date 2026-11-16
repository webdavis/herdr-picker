use std::path::PathBuf;

use pwe_application::{NoticeFloat, NoticeScreen};
use pwe_domain::{Corner, Level};
use serde_json::{Value, json};

use super::RexNoticeScreen;
use crate::cli_command::{RecordedCli, spawning};

const SESSION: &str = "session:00000000-0000-4000-8000-0000000000a1";
const FOCUSED: &str = "block:00000000-0000-4000-8000-0000000000b1";
const VIEW: &str = r#"{"revision":3,"session_id":"session:00000000-0000-4000-8000-0000000000a1","label":"invented","active_window_id":"window:00000000-0000-4000-8000-0000000000d1","windows":[{"window_id":"window:00000000-0000-4000-8000-0000000000d1","label":"invented","focused_block_id":"block:00000000-0000-4000-8000-0000000000b1","layers":[{"layer_id":"layer:00000000-0000-4000-8000-0000000000f1","kind":"tiled","bounds":[0,0,1,1],"layout":{"rect":[0,0,1,1],"block_id":"block:00000000-0000-4000-8000-0000000000b1"},"blocks":[{"block_id":"block:00000000-0000-4000-8000-0000000000b1","label":"shell","flavor":"com.superlogical.terminal.shell"}]}]}]}"#;
const SIZE: &str = r#"{"columns":154,"rows":107,"cell_width_px":8,"cell_height_px":16}"#;
const OPENED: &str = r#"{"layer_id":"layer:00000000-0000-4000-8000-0000000000f2","block_ids":["block:00000000-0000-4000-8000-0000000000b9"],"revision":4}"#;
const OPENED_LAYER: &str = "layer:00000000-0000-4000-8000-0000000000f2";
const OPENED_BLOCK: &str = "block:00000000-0000-4000-8000-0000000000b9";
const LINE: &str = "Tests passed, in tab api";

fn expected_request() -> Value {
    let width = 30.0 / 154.0;
    json!({
        "session_id": SESSION,
        "focus": false,
        "bounds": [1.0 - width, 0.0, width, 2.5 / 107.0],
        "layout": {"block": {
            "flavor": "com.superlogical.terminal.shell",
            "label": LINE,
            "options": {
                "size": {"columns": 30, "rows": 1},
                "command": ["/bin/pwe", "notice", "--level", "finished", "--until", "1790000106"],
            },
        }},
    })
}

fn rex_opening_one_float() -> RecordedCli {
    let view_request = json!({"session_id": SESSION}).to_string();
    let new_layer = expected_request().to_string();
    let relabel = json!({"session_id": SESSION, "block_id": OPENED_BLOCK, "label": "Tests failed: 3; $(touch nothing)"}).to_string();
    let close = json!({"session_id": SESSION, "layer_id": OPENED_LAYER}).to_string();
    RecordedCli::rex(&[
        (
            &format!("--autostart=false api call session.set_block_label {relabel}"),
            r#"{"revision":5}"#,
        ),
        (
            &format!("--autostart=false api call session.close_layer {close}"),
            r#"{"revision":6}"#,
        ),
        (
            &format!("--autostart=false api call session.view {view_request}"),
            VIEW,
        ),
        (
            &format!("--autostart=false block call -b {FOCUSED} size"),
            SIZE,
        ),
        (
            &format!("--autostart=false api call session.new_layer {new_layer}"),
            OPENED,
        ),
    ])
}

fn finished_notice() -> NoticeFloat {
    NoticeFloat {
        float: "float-1".to_string(),
        rex_session: SESSION.to_string(),
        slot: 0,
        level: Level::Finished,
        line: LINE.to_string(),
        until: 1790000106,
    }
}

fn screen(fake: &RecordedCli) -> RexNoticeScreen {
    RexNoticeScreen::at(
        PathBuf::from(&fake.binary),
        "/bin/pwe".to_string(),
        Corner::TopRight,
    )
}

#[test]
fn a_notice_opens_unfocused_in_the_top_right_slot() {
    let _spawning = spawning();
    let fake = rex_opening_one_float();

    screen(&fake).show(&finished_notice()).expect("shown");

    assert_eq!(
        fake.calls()[2],
        [
            "--autostart=false",
            "api",
            "call",
            "session.new_layer",
            &expected_request().to_string()
        ]
    );
}

#[test]
fn relabelling_passes_the_line_as_data_to_the_remembered_block() {
    let _spawning = spawning();
    let fake = rex_opening_one_float();
    let mut screen = screen(&fake);
    screen.show(&finished_notice()).expect("shown");

    screen
        .relabel("float-1", "Tests failed: 3; $(touch nothing)")
        .expect("relabelled");

    let request: Value = serde_json::from_str(&fake.calls()[3][4]).expect("JSON");
    assert_eq!(request["block_id"], OPENED_BLOCK);
    assert_eq!(request["label"], "Tests failed: 3; $(touch nothing)");
}

#[test]
fn closing_closes_the_remembered_layer_once() {
    let _spawning = spawning();
    let fake = rex_opening_one_float();
    let mut screen = screen(&fake);
    screen.show(&finished_notice()).expect("shown");

    screen.close("float-1").expect("closed");
    screen.close("float-1").expect("nothing left to close");

    let calls = fake.calls();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[3][3], "session.close_layer");
}

#[test]
fn an_unknown_float_is_left_alone() {
    let _spawning = spawning();
    let fake = rex_opening_one_float();
    let mut screen = screen(&fake);

    screen
        .relabel("float-9", "line")
        .expect("nothing to relabel");
    screen.close("float-9").expect("nothing to close");

    assert!(!fake.folder().join("count").exists());
}
