use super::*;

fn report(program: ProgramName, session_id: &str, pid: Option<u32>) -> ProgramReport {
    ProgramReport {
        program,
        session_id: session_id.to_string(),
        name: Some("tidy the release notes".to_string()),
        folder: Some("/work/project".to_string()),
        account: None,
        pid,
        state: AgentState::Idle,
        waiting_for: None,
        state_since: Some(100),
    }
}

fn pane(block: &str, program: ProgramName, pid: Option<u32>, state: AgentState) -> PaneStatus {
    PaneStatus {
        pane: Pane {
            rex_session: "session:one".to_string(),
            tab: "tab:one".to_string(),
            block: block.to_string(),
        },
        program: Some(program),
        pid,
        folder: Some("/work/project".to_string()),
        state,
        waiting_for: Some(WaitingFor {
            kind: WaitingKind::Permission,
            text: "approve Bash: ls".to_string(),
        }),
    }
}

fn blocks(book: &AgentBook) -> Vec<Option<String>> {
    book.sessions()
        .iter()
        .map(|session| session.pane.as_ref().map(|pane| pane.block.clone()))
        .collect()
}

#[test]
fn rex_state_wins_and_the_program_names_the_session() {
    let mut book = AgentBook::default();
    let claude = report(ProgramName::ClaudeCode, "22222222", Some(101));
    let rex = pane(
        "block:a",
        ProgramName::ClaudeCode,
        Some(101),
        AgentState::Waiting,
    );

    book.update(&[claude], &[rex], 200);

    let session = &book.sessions()[0];
    assert_eq!(session.session_id, "22222222");
    assert_eq!(session.name.as_deref(), Some("tidy the release notes"));
    assert_eq!(session.state, AgentState::Waiting);
    assert_eq!(session.state_from, StateFrom::Rex);
    assert_eq!(session.state_since, 200);
    assert_eq!(
        session.waiting_for.as_ref().map(|waiting| waiting.kind),
        Some(WaitingKind::Permission)
    );
    assert_eq!(blocks(&book), vec![Some("block:a".to_string())]);
}

#[test]
fn a_paneless_session_keeps_the_programs_state() {
    let mut book = AgentBook::default();

    let changes = book.update(&[report(ProgramName::Omp, "44444444", Some(102))], &[], 200);

    let session = &book.sessions()[0];
    assert_eq!(session.state, AgentState::Idle);
    assert_eq!(session.state_from, StateFrom::Program);
    assert_eq!(session.state_since, 100);
    assert_eq!(session.pane, None);
    assert_eq!(changes, vec![AgentChange::Started(session.clone())]);
}

#[test]
fn a_codex_thread_joins_only_the_one_pane_with_its_folder() {
    let thread = report(ProgramName::Codex, "33333333", None);
    let first = pane(
        "block:a",
        ProgramName::Codex,
        Some(109),
        AgentState::Working,
    );
    let second = pane(
        "block:b",
        ProgramName::Codex,
        Some(110),
        AgentState::Working,
    );
    let mut alone = AgentBook::default();
    let mut shared = AgentBook::default();

    alone.update(
        std::slice::from_ref(&thread),
        std::slice::from_ref(&first),
        200,
    );
    shared.update(&[thread], &[first, second], 200);

    assert_eq!(blocks(&alone), vec![Some("block:a".to_string())]);
    assert_eq!(blocks(&shared), vec![None]);
}

#[test]
fn a_vanished_pid_ends_the_session() {
    let mut book = AgentBook::default();
    book.update(
        &[report(ProgramName::ClaudeCode, "22222222", Some(101))],
        &[],
        200,
    );

    let changes = book.update(&[], &[], 300);

    let [AgentChange::Ended(ended)] = changes.as_slice() else {
        panic!("one ended session, got {changes:?}");
    };
    assert_eq!(ended.state, AgentState::Ended);
    assert_eq!(ended.state_since, 300);
    assert!(book.sessions().is_empty());
}

#[test]
fn an_unchanged_state_keeps_its_time_and_raises_no_change() {
    let mut book = AgentBook::default();
    let claude = report(ProgramName::ClaudeCode, "22222222", Some(101));
    let rex = pane(
        "block:a",
        ProgramName::ClaudeCode,
        Some(101),
        AgentState::Working,
    );
    book.update(
        std::slice::from_ref(&claude),
        std::slice::from_ref(&rex),
        200,
    );

    let changes = book.update(&[claude], &[rex], 260);

    assert!(changes.is_empty());
    assert_eq!(book.sessions()[0].state_since, 200);
}

#[test]
fn untrusted_names_and_reasons_lose_their_control_characters() {
    let mut book = AgentBook::default();
    let mut claude = report(ProgramName::ClaudeCode, "22222222", Some(101));
    claude.name = Some("tidy\u{1b}[2J the\nnotes".to_string());
    claude.state = AgentState::Waiting;
    claude.waiting_for = Some(WaitingFor {
        kind: WaitingKind::Other,
        text: "press\u{7} enter".to_string(),
    });

    book.update(&[claude], &[], 200);

    let session = &book.sessions()[0];
    assert_eq!(session.name.as_deref(), Some("tidy[2J thenotes"));
    assert_eq!(
        session
            .waiting_for
            .as_ref()
            .map(|waiting| waiting.text.as_str()),
        Some("press enter")
    );
}

#[test]
fn the_newest_state_comes_first() {
    let mut book = AgentBook::default();
    let mut older = report(ProgramName::Omp, "older", None);
    older.state_since = Some(50);
    let newer = report(ProgramName::Omp, "newer", None);

    book.update(&[older, newer], &[], 200);

    let order: Vec<String> = book
        .newest_state_first()
        .into_iter()
        .map(|session| session.session_id)
        .collect();
    assert_eq!(order, vec!["newer", "older"]);
}
