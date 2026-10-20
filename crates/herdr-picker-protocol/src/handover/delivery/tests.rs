use super::*;

const EMPTY: &str = "answer\n❯ \n";
const DRAFTED: &str = "answer\n❯ /Reply with\n";

#[test]
fn a_free_or_working_agent_with_an_empty_input_line_takes_it_now() {
    assert_eq!(
        ["idle", "done", "working"].map(|status| delivery(&PaneKind::Claude, status, EMPTY)),
        [Ok(Delivery::Now), Ok(Delivery::Now), Ok(Delivery::Now)]
    );
}

#[test]
fn a_blocked_agent_or_the_owners_draft_waits_until_it_is_free() {
    assert_eq!(
        (
            delivery(&PaneKind::Claude, "blocked", EMPTY),
            delivery(&PaneKind::Claude, "idle", DRAFTED),
            delivery(&PaneKind::Codex, "idle", "no prompt drawn"),
        ),
        (
            Ok(Delivery::WhenFree(Refusal::Blocked)),
            Ok(Delivery::WhenFree(Refusal::Draft(
                "/Reply with".to_string()
            ))),
            Ok(Delivery::WhenFree(Refusal::InputUnseen)),
        )
    );
}

#[test]
fn only_an_agent_herdr_picker_can_read_is_sent_to() {
    assert_eq!(
        (
            delivery(&PaneKind::Shell, "unknown", EMPTY),
            delivery(&PaneKind::Agent("hermes".to_string()), "idle", EMPTY),
        ),
        (
            Err(Refusal::UnknownAgent("a shell".to_string())),
            Err(Refusal::UnknownAgent("hermes".to_string())),
        )
    );
}
