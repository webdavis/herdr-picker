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
        parsed(&popup_request(
            "req_2",
            "herdr-picker",
            SIZE,
            &PopupPane::workspace_picker(Some("w6"))
        )),
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
    let request = parsed(&popup_request(
        "req_2",
        "herdr-picker",
        SIZE,
        &PopupPane::workspace_picker(None),
    ));

    assert_eq!(request["params"]["env"], json!({}));
}

#[test]
fn a_configured_picker_opens_in_the_callers_folder_and_is_told_what_to_run_and_where_to_answer() {
    let pane = PopupPane::configured_picker(
        "files",
        "/home/me/.config/herdr-picker/config.toml",
        "/home/me/.local/state/herdr/plugins/herdr-picker/picker-result-42",
        Some("/home/me/repos/netpulse"),
    );

    assert_eq!(
        parsed(&popup_request("req_2", "herdr-picker", SIZE, &pane)),
        json!({"id": "req_2", "method": "plugin.pane.open", "params": {
            "plugin_id": "herdr-picker",
            "entrypoint": "run_picker",
            "placement": "popup",
            "width": 129,
            "height": 40,
            "cwd": "/home/me/repos/netpulse",
            "env": {
                "HERDR_PICKER_NAME": "files",
                "HERDR_PICKER_CONFIG": "/home/me/.config/herdr-picker/config.toml",
                "HERDR_PICKER_RESULT": "/home/me/.local/state/herdr/plugins/herdr-picker/picker-result-42"
            }
        }})
    );
}

#[test]
fn a_configured_picker_without_a_known_folder_leaves_the_folder_to_herdr() {
    let pane = PopupPane::configured_picker("files", "/c.toml", "/r", None);

    let request = parsed(&popup_request("req_2", "herdr-picker", SIZE, &pane));

    assert_eq!(request["params"].get("cwd"), None);
}

#[test]
fn the_launcher_popup_carries_its_base_and_the_pane_under_it_and_opens_in_that_panes_folder() {
    let pane = PopupPane::launcher("ctrl-g", "w1:p3", Some("/work/repo"));

    assert_eq!(
        parsed(&popup_request("req_1", "herdr-picker", SIZE, &pane))["params"],
        json!({
            "plugin_id": "herdr-picker",
            "entrypoint": "launcher",
            "placement": "popup",
            "width": 129,
            "height": 40,
            "env": {"HERDR_PICKER_BASE": "ctrl-g", "HERDR_PICKER_TARGET": "w1:p3"},
            "cwd": "/work/repo"
        })
    );
}
