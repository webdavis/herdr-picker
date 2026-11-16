use pwe_protocol::NoticeRequest;

use super::*;

const NOW: u64 = 1_790_000_100;

fn asked(words: &[&str]) -> Option<NoticeRequest> {
    request(words, NOW).ok()
}

#[test]
fn notice_words_become_the_daemons_requests() {
    let id = || "1790000100-1".to_string();
    let source = || "tests".to_string();

    assert_eq!(asked(&["waiting"]), Some(NoticeRequest::Waiting));
    assert_eq!(asked(&["hide"]), Some(NoticeRequest::Hide));
    assert_eq!(asked(&["jump"]), Some(NoticeRequest::Jump { id: None }));
    assert_eq!(
        asked(&["jump", "1790000100-1"]),
        Some(NoticeRequest::Jump { id: Some(id()) })
    );
    assert_eq!(
        asked(&["dismiss", "1790000100-1"]),
        Some(NoticeRequest::Dismiss { id: id() })
    );
    assert_eq!(
        asked(&["read", "1790000100-1"]),
        Some(NoticeRequest::Read { id: id() })
    );
    assert_eq!(
        asked(&["mute", "tests"]),
        Some(NoticeRequest::Mute {
            source: source(),
            until: None
        })
    );
    assert_eq!(
        asked(&["unmute", "tests"]),
        Some(NoticeRequest::Unmute { source: source() })
    );
}

#[test]
fn mute_for_a_time_ends_that_long_from_now() {
    let until = |time: &str| match asked(&["mute", "tests", "--for", time]) {
        Some(NoticeRequest::Mute { until, .. }) => until,
        _ => panic!("{time} was refused"),
    };

    assert_eq!(
        [until("45s"), until("10m"), until("2h"), until("1d")],
        [
            Some(NOW + 45),
            Some(NOW + 600),
            Some(NOW + 7_200),
            Some(NOW + 86_400)
        ]
    );
}

#[test]
fn a_time_without_its_unit_or_a_missing_id_is_a_usage_error() {
    for words in [
        &["mute", "tests", "--for", "10"][..],
        &["mute", "tests", "--for", "m"][..],
        &["dismiss"][..],
        &["waiting", "now"][..],
    ] {
        assert!(
            matches!(request(words, NOW), Err(CommandError::Usage)),
            "{words:?}"
        );
    }
}

#[test]
fn with_the_daemon_stopped_a_notice_key_says_to_start_it() {
    let socket = std::env::temp_dir().join(format!("pwe-keys-{}.sock", std::process::id()));

    let refusal = ask(&socket, &NoticeRequest::Hide);

    assert!(
        matches!(&refusal, Err(CommandError::Failed(reason)) if reason.contains("pwe daemon start")),
        "{refusal:?}"
    );
}
