use super::*;
use pwe_domain::Pane;

fn recorded(program: ProgramName, session_id: &str) -> RecordedPane {
    RecordedPane {
        program,
        session_id: session_id.to_string(),
        folder: Some("/work/the project's notes".to_string()),
        pane: Pane {
            rex_session: "session:00000000-0000-7000-8000-0000000000a1".to_string(),
            tab: "window:00000000-0000-7000-8000-0000000000c1".to_string(),
            block: "block:00000000-0000-7000-8000-0000000000b1".to_string(),
        },
    }
}

const SESSION_ID: &str = "00000000-0000-4000-8000-000000000042";

#[test]
fn claude_code_resumes_its_session_by_its_id() {
    assert_eq!(
        resume_line(&recorded(ProgramName::ClaudeCode, SESSION_ID)),
        Ok(format!("claude --resume {SESSION_ID}"))
    );
}

#[test]
fn codex_resumes_with_its_resume_word() {
    let mut codex = recorded(ProgramName::Codex, SESSION_ID);
    codex.folder = None;

    assert_eq!(
        resume_line(&codex),
        Ok(format!("codex resume {SESSION_ID}"))
    );
}

#[test]
fn omp_has_no_resume_command() {
    assert_eq!(
        resume_line(&recorded(ProgramName::Omp, SESSION_ID)),
        Err(NoResumeLine::ProgramHasNone)
    );
}

fn refusal_for(session_id: &str, folder: &str) -> Result<String, NoResumeLine> {
    let mut claude = recorded(ProgramName::ClaudeCode, session_id);
    claude.folder = Some(folder.to_string());
    resume_line(&claude)
}

#[test]
fn a_session_id_with_a_semicolon_is_never_typed() {
    assert_eq!(
        refusal_for("42; rm -r ~", "/work"),
        Err(NoResumeLine::UnsafeSessionId)
    );
}

#[test]
fn a_session_id_with_a_control_character_is_never_typed() {
    assert_eq!(
        refusal_for("42\u{1b}[2J", "/work"),
        Err(NoResumeLine::UnsafeSessionId)
    );
}

#[test]
fn an_empty_or_overlong_session_id_is_never_typed() {
    assert_eq!(refusal_for("", "/work"), Err(NoResumeLine::UnsafeSessionId));
    assert_eq!(
        refusal_for(&"a".repeat(129), "/work"),
        Err(NoResumeLine::UnsafeSessionId)
    );
}

#[test]
fn a_session_id_of_letters_digits_dashes_and_underscores_is_typed() {
    assert!(refusal_for("Thread_01-ab", "/work").is_ok());
}

#[test]
fn the_resume_line_never_holds_the_folder() {
    assert_eq!(
        refusal_for(SESSION_ID, "/work\nrm -r ~"),
        Ok(format!("claude --resume {SESSION_ID}"))
    );
}

#[test]
fn recorded_panes_read_back_as_written() {
    let panes = vec![
        recorded(ProgramName::ClaudeCode, SESSION_ID),
        recorded(ProgramName::Codex, "11111111-1111-4111-8111-111111111111"),
    ];

    let written = agent_panes_json(&panes);

    assert_eq!(parse_agent_panes(&written), panes);
    assert!(written.contains("\"session-id\""), "{written}");
}

#[test]
fn an_unreadable_record_file_holds_no_panes() {
    assert_eq!(parse_agent_panes("not json"), Vec::new());
}
