use pwe_domain::{OwnerPlace, SessionSeen};
use serde::Deserialize;

#[derive(Deserialize)]
struct Reported {
    running: bool,
    #[serde(default)]
    frontmost: bool,
    session: Option<ReportedSession>,
}

#[derive(Deserialize)]
struct ReportedSession {
    id: String,
    name: String,
    remote: bool,
}

fn without_host(id: &str) -> &str {
    id.rsplit_once('/').map_or(id, |(_, session)| session)
}

fn seen(session: Option<ReportedSession>) -> SessionSeen {
    match session {
        None => SessionSeen::NoSession,
        Some(session) if session.remote => SessionSeen::Elsewhere { name: session.name },
        Some(session) => SessionSeen::OnThisMac {
            rex_session: without_host(&session.id).to_string(),
            name: session.name,
        },
    }
}

pub fn parse_rex_whereabouts(text: &str) -> Result<OwnerPlace, serde_json::Error> {
    let reported: Reported = serde_json::from_str(text.trim())?;
    Ok(match reported {
        Reported { running: false, .. } => OwnerPlace::Closed,
        Reported {
            frontmost: true,
            session,
            ..
        } => OwnerPlace::Front(seen(session)),
        Reported { session, .. } => OwnerPlace::Background(seen(session)),
    })
}
