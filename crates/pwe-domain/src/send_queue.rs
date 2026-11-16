// A prompt waiting for its agent to finish its turn with an empty input line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queued {
    pub pane_id: String,
    pub text: String,
}

// The pane's oldest waiting prompt, and the queue without it; one a turn, since sending starts one.
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
