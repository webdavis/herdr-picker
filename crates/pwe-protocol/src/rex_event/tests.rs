use super::*;

const RECORDED: &str = include_str!("fixtures/rex-events-all.jsonl");

const FIRST_SESSION: &str = "session:00000000-0000-7000-8000-00000000a001";
const SECOND_SESSION: &str = "session:00000000-0000-7000-8000-00000000a002";
const AGENT_BLOCK: &str = "block:00000000-0000-7000-8000-00000000d001";
const SHELL_BLOCK: &str = "block:00000000-0000-7000-8000-00000000d002";

fn recorded_line(number: usize) -> &'static str {
    RECORDED
        .lines()
        .nth(number - 1)
        .unwrap_or_else(|| panic!("the fixture has no line {number}"))
}

fn event_on_line(number: usize) -> RexEvent {
    parse_rex_event(number, recorded_line(number))
        .unwrap_or_else(|refusal| panic!("line {number} was refused: {refusal}"))
}

fn at(rex_session: &str, block: &str) -> RexBlock {
    RexBlock {
        rex_session: rex_session.to_string(),
        block: block.to_string(),
    }
}

#[test]
fn a_tab_change_names_its_session_and_tab() {
    assert_eq!(
        event_on_line(2),
        RexEvent::TabChanged {
            rex_session: FIRST_SESSION.to_string(),
            tab: "window:00000000-0000-7000-8000-00000000b001".to_string(),
        }
    );
}

#[test]
fn a_view_change_names_its_session() {
    assert_eq!(
        event_on_line(3),
        RexEvent::ViewChanged {
            rex_session: FIRST_SESSION.to_string(),
        }
    );
}

#[test]
fn a_blocked_program_status_keeps_its_kind_message_and_pid() {
    assert_eq!(
        event_on_line(4),
        RexEvent::ProgramStatusChanged {
            at: at(SECOND_SESSION, AGENT_BLOCK),
            status: ProgramStatus {
                state: "blocked".to_string(),
                kind: Some("permission".to_string()),
                program: "example-agent".to_string(),
                message: Some("approve an example step".to_string()),
                pid: Some(2222),
            },
        }
    );
}

#[test]
fn a_working_program_status_has_no_kind_or_message() {
    let RexEvent::ProgramStatusChanged { status, .. } = event_on_line(5) else {
        panic!("line 5 is a program status change");
    };

    assert_eq!(
        (status.state.as_str(), status.kind, status.message),
        ("working", None, None)
    );
}

#[test]
fn a_removed_program_status_names_its_block() {
    assert_eq!(
        event_on_line(6),
        RexEvent::ProgramStatusRemoved {
            at: at(SECOND_SESSION, AGENT_BLOCK),
        }
    );
}

#[test]
fn a_process_change_names_the_foreground_program_and_pid() {
    assert_eq!(
        event_on_line(7),
        RexEvent::ProcessChanged {
            at: at(SECOND_SESSION, SHELL_BLOCK),
            foreground: Some(ForegroundProcess {
                pid: 3333,
                name: "example-shell".to_string(),
            }),
        }
    );
}

#[test]
fn a_child_exit_keeps_its_exit_code() {
    assert_eq!(
        event_on_line(9),
        RexEvent::ChildExited {
            at: at(SECOND_SESSION, SHELL_BLOCK),
            exit_code: Some(3),
        }
    );
}

#[test]
fn a_closed_block_names_its_session_and_block() {
    assert_eq!(
        event_on_line(10),
        RexEvent::BlockClosed {
            at: at(SECOND_SESSION, SHELL_BLOCK),
        }
    );
}

#[test]
fn events_pwe_does_not_follow_read_as_other() {
    for number in [1, 8, 11] {
        assert_eq!(event_on_line(number), RexEvent::Other, "line {number}");
    }
}

#[test]
fn an_unknown_event_type_reads_as_other() {
    let unknown = r#"{"type":"session_event","payload":{"session_id":"session:x","event":{"type":"something_new","payload":{}}}}"#;

    assert_eq!(parse_rex_event(1, unknown), Ok(RexEvent::Other));
}

#[test]
fn a_line_that_is_not_json_is_refused_naming_its_line() {
    let refusal =
        parse_rex_event(12, recorded_line(12)).expect_err("a line that is not JSON is refused");

    assert_eq!(refusal.line_number, 12);
    assert!(
        refusal.to_string().contains("line 12")
            && refusal.to_string().contains("Error: interrupted"),
        "{refusal}"
    );
}
