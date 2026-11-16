use pwe_domain::{AgentState, ProgramName};

use super::*;

struct Listing(Result<Vec<ProgramReport>, AgentReadError>);

impl AgentProgram for Listing {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError> {
        self.0.clone()
    }
}

fn report(session_id: &str, state_since: u64) -> ProgramReport {
    ProgramReport {
        program: ProgramName::Omp,
        session_id: session_id.to_string(),
        name: None,
        folder: None,
        account: None,
        pid: None,
        state: AgentState::Idle,
        waiting_for: None,
        state_since: Some(state_since),
    }
}

#[test]
fn every_program_is_read_and_the_newest_state_comes_first() {
    let mut first = Listing(Ok(vec![report("older", 10)]));
    let mut second = Listing(Ok(vec![report("newer", 20)]));

    let (sessions, refusals) = agent_sessions(&mut [&mut first, &mut second], 30);

    let order: Vec<&str> = sessions.iter().map(|s| s.session_id.as_str()).collect();
    assert_eq!(order, ["newer", "older"]);
    assert!(refusals.is_empty());
}

#[test]
fn a_program_that_fails_leaves_the_others_listed() {
    let refusal = AgentReadError {
        program: "claude-code".to_string(),
        detail: "exited 1".to_string(),
    };
    let mut failing = Listing(Err(refusal.clone()));
    let mut working = Listing(Ok(vec![report("one", 10)]));

    let (sessions, refusals) = agent_sessions(&mut [&mut failing, &mut working], 30);

    assert_eq!(sessions.len(), 1);
    assert_eq!(refusals, vec![refusal]);
}
