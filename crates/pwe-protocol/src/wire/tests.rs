use super::*;

fn parsed(line: &str) -> Value {
    match serde_json::from_str(line.trim()) {
        Ok(value) => value,
        Err(error) => panic!("encoder produced invalid JSON: {error}"),
    }
}

#[test]
fn every_request_is_one_json_line_terminated_by_a_newline() {
    for line in [
        list_request("1"),
        focus_request("2", "wA"),
        create_request("3", "dotfiles", "/tmp/x"),
    ] {
        assert!(line.ends_with('\n'), "missing newline: {line}");
        assert_eq!(line.matches('\n').count(), 1, "embedded newline: {line}");
        assert!(parsed(&line).is_object());
    }
}

#[test]
fn list_request_names_the_method_and_carries_the_id() {
    let request = parsed(&list_request("req_1"));
    assert_eq!(request["method"], "workspace.list");
    assert_eq!(request["id"], "req_1");
}

#[test]
fn focus_request_carries_the_workspace_id() {
    let request = parsed(&focus_request("req_2", "w11"));
    assert_eq!(request["method"], "workspace.focus");
    assert_eq!(request["params"]["workspace_id"], "w11");
}

#[test]
fn create_request_carries_cwd_and_label_and_focuses_the_workspace_it_creates_to_jump_into() {
    let request = parsed(&create_request("req_3", "netpulse", "/home/me/netpulse"));
    assert_eq!(request["method"], "workspace.create");
    assert_eq!(request["params"]["cwd"], "/home/me/netpulse");
    assert_eq!(request["params"]["label"], "netpulse");
    assert_eq!(request["params"]["focus"], true);
}

#[test]
fn create_request_escapes_a_label_that_would_break_the_line() {
    let request = create_request("4", "odd \"quoted\"\nlabel", "/tmp");
    assert_eq!(request.matches('\n').count(), 1);
    assert_eq!(parsed(&request)["params"]["label"], "odd \"quoted\"\nlabel");
}

#[test]
fn result_of_unwraps_a_success_envelope() {
    let result = result_of(r#"{"id":"1","result":{"type":"pong"}}"#);
    assert_eq!(result, Ok(json!({"type": "pong"})));
}

#[test]
fn result_of_rejects_an_empty_socket_body() {
    assert!(matches!(result_of(""), Err(ProtocolError::Malformed(_))));
    assert!(matches!(
        result_of("  \n "),
        Err(ProtocolError::Malformed(_))
    ));
}

#[test]
fn result_of_reports_an_error_envelope_as_a_server_failure() {
    let failure =
        result_of(r#"{"id":"1","error":{"code":"not_found","message":"no such workspace"}}"#);
    assert_eq!(
        failure,
        Err(ProtocolError::Server {
            code: "not_found".to_string(),
            message: "no such workspace".to_string(),
        })
    );
}

#[test]
fn result_of_rejects_non_json_and_an_envelope_with_neither_arm() {
    assert!(matches!(
        result_of("not json"),
        Err(ProtocolError::Malformed(_))
    ));
    assert!(matches!(
        result_of(r#"{"id":"1"}"#),
        Err(ProtocolError::Malformed(_))
    ));
}

#[test]
fn workspaces_of_reads_id_and_label() {
    let result = json!({"workspaces": [
        {"workspace_id": "wW", "label": "dotfiles modernization"},
        {"workspace_id": "w11", "label": "dotfiles"}
    ]});
    assert_eq!(
        workspaces_of(&result),
        Ok(vec![
            WireWorkspace {
                workspace_id: "wW".to_string(),
                label: "dotfiles modernization".to_string(),
            },
            WireWorkspace {
                workspace_id: "w11".to_string(),
                label: "dotfiles".to_string(),
            },
        ])
    );
}

#[test]
fn a_labelless_record_is_kept_with_an_empty_label_that_no_jump_can_match() {
    let result =
        json!({"workspaces": [{"workspace_id": "wA"}, {"workspace_id": "wB", "label": null}]});
    let workspaces = match workspaces_of(&result) {
        Ok(workspaces) => workspaces,
        Err(error) => panic!("expected the records to parse: {error:?}"),
    };
    assert_eq!(workspaces.len(), 2);
    assert!(
        workspaces
            .iter()
            .all(|workspace| workspace.label.is_empty())
    );
}

#[test]
fn workspaces_of_rejects_a_missing_array_or_a_record_without_an_id() {
    assert!(matches!(
        workspaces_of(&json!({})),
        Err(ProtocolError::Malformed(_))
    ));
    assert!(matches!(
        workspaces_of(&json!({"workspaces": [{"label": "x"}]})),
        Err(ProtocolError::Malformed(_))
    ));
}
