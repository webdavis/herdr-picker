use super::{PaneKind, Refusal, guard};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    Now,
    WhenFree(Refusal),
}

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
