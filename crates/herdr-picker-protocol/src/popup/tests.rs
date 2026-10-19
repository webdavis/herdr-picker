use super::*;

fn parsed(line: &str) -> Value {
    assert!(line.ends_with('\n'), "missing newline: {line}");
    match serde_json::from_str(line.trim()) {
        Ok(value) => value,
        Err(error) => panic!("encoder produced invalid JSON: {error}"),
    }
}

const SIZE: Size = Size {
    columns: 129,
    rows: 40,
};

#[test]
fn the_layout_request_asks_for_the_focused_panes_layout() {
    assert_eq!(
        parsed(&layout_request("req_1")),
        json!({"id": "req_1", "method": "pane.layout", "params": {}})
    );
}

#[test]
fn the_window_is_the_area_of_the_layout() {
    let result = json!({"type": "pane_layout", "layout": {
        "workspace_id": "w6",
        "area": {"x": 0, "y": 0, "width": 175, "height": 65},
        "panes": [{"pane_id": "w6:p1", "rect": {"x": 0, "y": 0, "width": 88, "height": 65}}]
    }});

    assert_eq!(
        window_of(&result),
        Ok(Size {
            columns: 175,
            rows: 65
        })
    );
}

#[test]
fn a_layout_without_an_area_size_is_malformed() {
    for result in [
        json!({"type": "pane_layout", "layout": {}}),
        json!({"type": "pane_layout", "layout": {"area": {"width": 175}}}),
        json!({"type": "pane_layout", "layout": {"area": {"width": "wide", "height": 65}}}),
    ] {
        assert!(
            matches!(window_of(&result), Err(ProtocolError::Malformed(_))),
            "{result}"
        );
    }
}

#[test]
fn the_picker_request_opens_the_pick_pane_as_a_popup_for_the_current_workspace() {
    assert_eq!(
        parsed(&picker_request("req_2", "herdr-picker", SIZE, Some("w6"))),
        json!({"id": "req_2", "method": "plugin.pane.open", "params": {
            "plugin_id": "herdr-picker",
            "entrypoint": "pick",
            "placement": "popup",
            "width": 129,
            "height": 40,
            "env": {"HERDR_ACTIVE_WORKSPACE_ID": "w6"}
        }})
    );
}

#[test]
fn without_a_current_workspace_the_popup_is_handed_none() {
    let request = parsed(&picker_request("req_2", "herdr-picker", SIZE, None));

    assert_eq!(request["params"]["env"], json!({}));
}
