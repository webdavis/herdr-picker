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

const EMPTY_STARSHIP: &str = "~/repo on main\n❯ ls\nsrc\n\n~/repo on main\n❯ \n\n\n";

#[test]
fn a_shell_whose_prompt_line_is_empty_may_be_typed_into() {
    assert_eq!(guard(&PaneKind::Shell, "unknown", EMPTY_STARSHIP), Ok(()));
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "out\nme@box:~$ "),
        Ok(())
    );
    assert_eq!(guard(&PaneKind::Shell, "unknown", "~/repo\n❮ "), Ok(()));
}

#[test]
fn a_shell_draft_after_the_prompt_is_never_typed_over() {
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "~/repo on main\n❯ git sta\n"),
        Err(Refusal::Draft("git sta".to_string()))
    );
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "me@box:~$ make te"),
        Err(Refusal::Draft("make te".to_string()))
    );
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "❯ echo $"),
        Err(Refusal::Draft("echo $".to_string()))
    );
}

#[test]
fn a_shell_whose_prompt_cannot_be_found_is_left_alone() {
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "~\n:wq"),
        Err(Refusal::InputUnseen)
    );
}

#[test]
fn the_owners_prompt_is_read_with_and_without_a_draft() {
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "~/repo main\n+❯ "),
        Ok(())
    );
    assert_eq!(
        guard(&PaneKind::Shell, "unknown", "~/repo main\n+❯ git di"),
        Err(Refusal::Draft("git di".to_string()))
    );
}

#[test]
fn a_program_running_in_the_shell_is_never_mistaken_for_its_prompt() {
    for screen in [
        "fn main() {}\n~\nsrc/main.rs [+]  12,5  50%",
        "Python 3.13.0\n>>> ",
        "Compiling\nBuilding... 3/10 >",
        "Password: ",
    ] {
        assert_eq!(
            guard(&PaneKind::Shell, "unknown", screen),
            Err(Refusal::InputUnseen),
            "{screen}"
        );
    }
}
