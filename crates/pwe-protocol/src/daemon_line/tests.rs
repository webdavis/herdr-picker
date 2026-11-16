use super::*;
use serde_json::json;

fn refusal(line: &str) -> DaemonError {
    match parse_daemon_request(line) {
        Ok(request) => panic!("{line} was taken as {request:?}"),
        Err(refusal) => refusal,
    }
}

#[test]
fn an_unknown_request_is_refused_by_name() {
    let refused = refusal(r#"{"pwe-daemon":1,"request":"frobnicate"}"#);

    assert_eq!(refused.kind, DaemonErrorKind::UnknownRequest);
    assert!(
        refused.message.contains("\"frobnicate\""),
        "{}",
        refused.message
    );
}

#[test]
fn a_request_line_reads_back_as_the_same_request() {
    for request in [DaemonRequest::Status, DaemonRequest::Stop] {
        let line = daemon_request_line(request);

        assert!(
            line.ends_with('\n') && line.matches('\n').count() == 1,
            "{line}"
        );
        assert_eq!(parse_daemon_request(&line), Ok(request));
    }
}

#[test]
fn a_status_request_carries_the_protocol_version_and_its_word() {
    let line: serde_json::Value = serde_json::from_str(&daemon_request_line(DaemonRequest::Status))
        .unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(line, json!({"pwe-daemon": 1, "request": "status"}));
}

#[test]
fn a_missing_request_or_another_version_is_a_bad_field() {
    for line in [
        r#"{"pwe-daemon":1}"#,
        r#"{"pwe-daemon":2,"request":"status"}"#,
        r#"{"request":"status"}"#,
        r#"{"pwe-daemon":1,"request":7}"#,
    ] {
        assert_eq!(refusal(line).kind, DaemonErrorKind::BadField, "{line}");
    }
}

#[test]
fn a_line_that_is_not_a_json_object_is_a_bad_line() {
    for line in ["status", "[1]", ""] {
        assert_eq!(refusal(line).kind, DaemonErrorKind::BadLine, "{line:?}");
    }
}

#[test]
fn a_reply_line_reads_back_as_its_result_or_its_error() {
    let answered: Result<serde_json::Value, DaemonError> = Ok(json!({"running": true}));
    let refused: Result<serde_json::Value, DaemonError> =
        Err(refusal(r#"{"pwe-daemon":1,"request":"x"}"#));

    for reply in [answered, refused] {
        let line = daemon_reply_line(&reply);
        assert!(
            line.ends_with('\n') && line.matches('\n').count() == 1,
            "{line}"
        );
        assert_eq!(parse_daemon_reply(&line), reply);
    }
}

#[test]
fn a_refusal_reply_uses_the_kebab_case_kind() {
    let line = daemon_reply_line(&Err(refusal(r#"{"pwe-daemon":1,"request":"x"}"#)));
    let reply: serde_json::Value =
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(reply["ok"], json!(false));
    assert_eq!(reply["error"]["kind"], json!("unknown-request"));
}

#[test]
fn an_unreadable_reply_is_named_as_such() {
    let read = parse_daemon_reply("not json");

    assert!(matches!(
        read,
        Err(DaemonError {
            kind: DaemonErrorKind::UnreadableReply,
            ..
        })
    ));
}
