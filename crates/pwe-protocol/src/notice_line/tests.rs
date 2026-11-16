use super::*;
use crate::DaemonErrorKind;
use serde_json::json;

fn notify() -> NotifyRequest {
    NotifyRequest {
        source: "tests".to_string(),
        kind: "run".to_string(),
        key: "tests-api".to_string(),
        level: Level::Finished,
        text: "Tests passed".to_string(),
        pane: Some("block:00000000-0000-4000-8000-0000000000b1".to_string()),
    }
}

fn parsed(line: &str) -> Result<NoticeRequest, DaemonError> {
    parse_notice_request(line).unwrap_or_else(|| panic!("{line} was not taken as a notice request"))
}

#[test]
fn a_notify_line_reads_back_as_the_same_request() {
    let request = NoticeRequest::Notify(notify());

    assert_eq!(parsed(&notice_request_line(&request)), Ok(request));
}

#[test]
fn a_notify_line_names_its_fields_in_the_command_lines_words() {
    let line: serde_json::Value =
        serde_json::from_str(&notice_request_line(&NoticeRequest::Notify(notify())))
            .unwrap_or_else(|failure| panic!("{failure}"));

    assert_eq!(
        line,
        json!({
            "pwe-daemon": 1,
            "request": "notify",
            "source": "tests",
            "kind": "run",
            "key": "tests-api",
            "level": "finished",
            "text": "Tests passed",
            "pane": "block:00000000-0000-4000-8000-0000000000b1",
        })
    );
}

#[test]
fn a_resolve_line_reads_back_as_the_same_request() {
    let request = NoticeRequest::Resolve {
        key: "tests-api".to_string(),
    };

    assert_eq!(parsed(&notice_request_line(&request)), Ok(request));
}

#[test]
fn control_characters_are_stripped_and_long_text_is_cut() {
    let long = "x".repeat(400);
    let line = json!({"pwe-daemon": 1, "request": "notify", "source": "te\u{1b}[31msts",
        "kind": "run", "key": "k", "level": "quiet", "text": format!("a\nb\u{7}{long}")})
    .to_string();

    let Ok(NoticeRequest::Notify(request)) = parsed(&line) else {
        panic!("{line} was refused");
    };

    assert_eq!(request.source, "te[31msts");
    assert!(request.text.starts_with("abx"), "{}", request.text);
    assert!(request.text.chars().count() <= 128, "{}", request.text);
    assert!(!request.text.chars().any(char::is_control));
}

#[test]
fn an_unknown_level_is_a_bad_field_naming_the_accepted_words() {
    let line = json!({"pwe-daemon": 1, "request": "notify", "source": "tests",
        "kind": "run", "key": "k", "level": "urgent", "text": "t"})
    .to_string();

    let refusal = parsed(&line).err();

    assert_eq!(
        refusal.as_ref().map(|refusal| refusal.kind),
        Some(DaemonErrorKind::BadField)
    );
    assert!(refusal.is_some_and(|refusal| refusal.message.contains("needs-you, finished, quiet")),);
}

#[test]
fn a_notify_without_a_source_is_a_bad_field() {
    let line = json!({"pwe-daemon": 1, "request": "notify", "kind": "run", "key": "k",
        "level": "quiet", "text": "t"})
    .to_string();

    assert_eq!(
        parsed(&line).map_err(|refusal| refusal.kind),
        Err(DaemonErrorKind::BadField)
    );
}

#[test]
fn a_word_the_notices_do_not_know_is_left_to_the_daemon() {
    assert_eq!(
        parse_notice_request(r#"{"pwe-daemon":1,"request":"status"}"#),
        None
    );
}

#[test]
fn every_notice_key_line_reads_back_as_the_same_request() {
    let id = || "1790000100-1".to_string();
    for request in [
        NoticeRequest::Waiting,
        NoticeRequest::Hide,
        NoticeRequest::Jump { id: None },
        NoticeRequest::Jump { id: Some(id()) },
        NoticeRequest::Dismiss { id: id() },
        NoticeRequest::Read { id: id() },
        NoticeRequest::Mute {
            source: "tests".to_string(),
            until: Some(1_790_000_700),
        },
        NoticeRequest::Mute {
            source: "*".to_string(),
            until: None,
        },
        NoticeRequest::Unmute {
            source: "tests".to_string(),
        },
    ] {
        assert_eq!(parsed(&notice_request_line(&request)), Ok(request));
    }
}

#[test]
fn the_notice_key_words_are_the_designs() {
    let word = |request: NoticeRequest| -> String {
        let line: serde_json::Value = serde_json::from_str(&notice_request_line(&request))
            .unwrap_or_else(|failure| panic!("{failure}"));
        line["request"].as_str().unwrap_or_default().to_string()
    };

    assert_eq!(
        [
            word(NoticeRequest::Waiting),
            word(NoticeRequest::Hide),
            word(NoticeRequest::Jump { id: None }),
            word(NoticeRequest::Dismiss {
                id: "i".to_string()
            }),
            word(NoticeRequest::Read {
                id: "i".to_string()
            }),
            word(NoticeRequest::Unmute {
                source: "s".to_string()
            }),
        ],
        [
            "notice-waiting",
            "notice-hide",
            "notice-jump",
            "notice-dismiss",
            "notice-read",
            "unmute"
        ]
    );
}

#[test]
fn a_mute_until_that_is_not_a_time_is_a_bad_field() {
    let line = r#"{"pwe-daemon":1,"request":"mute","source":"tests","until":"soon"}"#;

    assert_eq!(
        parsed(line).map_err(|refusal| refusal.kind),
        Err(DaemonErrorKind::BadField)
    );
}
