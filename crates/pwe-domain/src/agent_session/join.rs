use super::{
    AgentSession, AgentState, PaneStatus, ProgramName, ProgramReport, StateFrom, WaitingFor,
};

pub(super) fn joined(
    report: &ProgramReport,
    panes: &[PaneStatus],
    earlier: Option<&AgentSession>,
    now: u64,
) -> AgentSession {
    let pane = pane_of(report, panes);
    let (state, waiting_for, state_from) = match pane {
        Some(pane) if pane.state != AgentState::Unknown => {
            (pane.state, pane.waiting_for.clone(), StateFrom::Rex)
        }
        _ => (report.state, report.waiting_for.clone(), StateFrom::Program),
    };
    let waiting_for = waiting_for.map(|waiting| WaitingFor {
        kind: waiting.kind,
        text: printable(&waiting.text),
    });
    let state_since = match earlier {
        Some(earlier) if earlier.state == state && earlier.waiting_for == waiting_for => {
            earlier.state_since
        }
        _ if state_from == StateFrom::Program => report.state_since.unwrap_or(now),
        _ => now,
    };
    AgentSession {
        program: report.program,
        session_id: report.session_id.clone(),
        name: report.name.as_deref().map(printable),
        folder: report.folder.clone(),
        account: report.account.clone(),
        pid: report.pid,
        pane: pane.map(|pane| pane.pane.clone()),
        state,
        waiting_for,
        state_since,
        state_from,
    }
}

fn pane_of<'a>(report: &ProgramReport, panes: &'a [PaneStatus]) -> Option<&'a PaneStatus> {
    if report.program == ProgramName::Codex {
        return only_pane_in_the_folder(report, panes);
    }
    let pid = report.pid?;
    panes.iter().find(|pane| pane.pid == Some(pid))
}

fn only_pane_in_the_folder<'a>(
    report: &ProgramReport,
    panes: &'a [PaneStatus],
) -> Option<&'a PaneStatus> {
    let folder = report.folder.as_ref()?;
    let mut matching = panes.iter().filter(|pane| {
        pane.program == Some(ProgramName::Codex) && pane.folder.as_ref() == Some(folder)
    });
    let only = matching.next()?;
    matching.next().is_none().then_some(only)
}

fn printable(text: &str) -> String {
    text.chars().filter(|glyph| !glyph.is_control()).collect()
}
