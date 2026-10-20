use super::*;

#[test]
fn parse_focused_id_reads_the_workspace_id_and_survives_garbage() {
    let event = r#"{"event":"workspace_focused","data":{"workspace_id":"w18"}}"#;
    assert_eq!(parse_focused_id(event), Some("w18".to_string()));
    assert_eq!(parse_focused_id("not json"), None);
    assert_eq!(parse_focused_id("{}"), None);
}

#[test]
fn a_pane_focus_names_the_pane_and_a_workspace_focus_does_not() {
    let pane = r#"{"event":"pane_focused","data":{"pane_id":"w1:p3","workspace_id":"w1"}}"#;
    let workspace = r#"{"event":"workspace_focused","data":{"workspace_id":"w1"}}"#;

    assert_eq!(parse_focused_pane(pane), Some("w1:p3".to_string()));
    assert_eq!(parse_focused_pane(workspace), None);
}

#[test]
fn a_plugin_action_finds_the_pane_under_it_and_its_folder_in_its_context() {
    let context =
        r#"{"focused_pane_id":"w1:p3","focused_pane_cwd":"/work","focused_pane_agent":"claude"}"#;

    assert_eq!(
        parse_context_pane(context),
        Some(("w1:p3".to_string(), Some("/work".to_string())))
    );
    assert_eq!(
        parse_context_pane(r#"{"focused_pane_id":"w1:p3"}"#),
        Some(("w1:p3".to_string(), None))
    );
    assert_eq!(parse_context_pane(r#"{"workspace_id":"w1"}"#), None);
}
