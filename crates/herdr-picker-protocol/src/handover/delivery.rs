use super::{PaneKind, Refusal, guard};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    // herdr holds a prompt sent while the agent works and hands it over after the turn.
    Now,
    WhenFree(Refusal),
}

// Whether a prompt goes to an agent now or waits in the queue; the owner's draft is never touched.
pub fn delivery(kind: &PaneKind, status: &str, screen: &str) -> Result<Delivery, Refusal> {
    match kind {
        PaneKind::Shell => return Err(Refusal::UnknownAgent("a shell".to_string())),
        PaneKind::Agent(name) => return Err(Refusal::UnknownAgent(name.clone())),
        PaneKind::Claude | PaneKind::Codex => {}
    }
    Ok(match guard(kind, status, screen) {
        Ok(()) => Delivery::Now,
        Err(waiting) => Delivery::WhenFree(waiting),
    })
}

#[cfg(test)]
mod tests;
