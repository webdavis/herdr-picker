#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queued {
    pub pane_id: String,
    pub text: String,
}

pub fn take_next(queue: Vec<Queued>, pane_id: &str) -> (Option<Queued>, Vec<Queued>) {
    let mut rest = queue;
    let next = rest
        .iter()
        .position(|queued| queued.pane_id == pane_id)
        .map(|index| rest.remove(index));
    (next, rest)
}

#[cfg(test)]
mod tests;
