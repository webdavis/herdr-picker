use super::*;

const EMPTY_CLAUDE: &str =
    "  Done.\n─────────\n❯ \n─────────\n  -- INSERT -- ⏵⏵ bypass permissions on\n";
const DRAFT_CLAUDE: &str = "  Done.\n─────────\n❯ /Reply with the\n─────────\n  -- INSERT --\n";
const EMPTY_CODEX: &str = "• Ran tests\n\n› \n\n  ? for shortcuts\n";

#[test]
fn an_agent_with_an_empty_input_line_may_be_typed_into() {
    assert_eq!(guard(&PaneKind::Claude, "idle", EMPTY_CLAUDE), Ok(()));
    assert_eq!(guard(&PaneKind::Claude, "working", EMPTY_CLAUDE), Ok(()));
    assert_eq!(guard(&PaneKind::Codex, "done", EMPTY_CODEX), Ok(()));
}

#[test]
fn the_owners_draft_is_never_typed_over() {
    assert_eq!(
        guard(&PaneKind::Claude, "done", DRAFT_CLAUDE),
        Err(Refusal::Draft("/Reply with the".to_string()))
    );
}

#[test]
fn an_agent_waiting_on_a_question_or_whose_input_line_cannot_be_seen_is_left_alone() {
    assert_eq!(
        guard(&PaneKind::Claude, "blocked", EMPTY_CLAUDE),
        Err(Refusal::Blocked)
    );
    assert_eq!(
        guard(&PaneKind::Codex, "idle", EMPTY_CLAUDE),
        Err(Refusal::InputUnseen)
    );
}

#[test]
fn a_shell_is_typed_into_as_its_own_key_would() {
    assert_eq!(guard(&PaneKind::Shell, "unknown", "$ git sta"), Ok(()));
}
