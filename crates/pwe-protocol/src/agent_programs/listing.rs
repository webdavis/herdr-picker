use pwe_domain::{AgentSession, Pane, WaitingFor};
use serde::Serialize;

#[derive(Serialize)]
struct Listing<'a> {
    #[serde(rename = "agent-sessions")]
    agent_sessions: Vec<ListedSession<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ListedSession<'a> {
    program: &'static str,
    session_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pane: Option<ListedPane<'a>>,
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    waiting_for: Option<ListedWait<'a>>,
    state_since: u64,
    state_from: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ListedPane<'a> {
    rex_session: &'a str,
    tab: &'a str,
    block: &'a str,
}

#[derive(Serialize)]
struct ListedWait<'a> {
    kind: &'static str,
    text: &'a str,
}

pub fn agent_sessions_json(sessions: &[AgentSession]) -> serde_json::Result<String> {
    serde_json::to_string(&Listing {
        agent_sessions: sessions.iter().map(listed).collect(),
    })
}

fn listed(session: &AgentSession) -> ListedSession<'_> {
    ListedSession {
        program: session.program.word(),
        session_id: &session.session_id,
        name: session.name.as_deref(),
        folder: session.folder.as_deref(),
        account: session.account.as_deref(),
        pid: session.pid,
        pane: session.pane.as_ref().map(listed_pane),
        state: session.state.word(),
        waiting_for: session.waiting_for.as_ref().map(listed_wait),
        state_since: session.state_since,
        state_from: session.state_from.word(),
    }
}

fn listed_pane(pane: &Pane) -> ListedPane<'_> {
    ListedPane {
        rex_session: &pane.rex_session,
        tab: &pane.tab,
        block: &pane.block,
    }
}

fn listed_wait(waiting: &WaitingFor) -> ListedWait<'_> {
    ListedWait {
        kind: waiting.kind.word(),
        text: &waiting.text,
    }
}

#[cfg(test)]
mod tests;
