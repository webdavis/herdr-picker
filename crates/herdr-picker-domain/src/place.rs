// A pane herdr focused, where it was, and when, in seconds since the epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub at: u64,
    pub workspace_id: String,
    pub tab_id: String,
    pub pane_id: String,
    pub cwd: String,
}

pub const RECENT_PLACES: usize = 200;

// Oldest first, each pane and folder once, at most `bound` of them.
pub fn remember(mut places: Vec<Place>, focused: Place, bound: usize) -> Vec<Place> {
    places.retain(|place| (&place.pane_id, &place.cwd) != (&focused.pane_id, &focused.cwd));
    places.push(focused);
    let excess = places.len().saturating_sub(bound);
    places.drain(..excess);
    places
}

#[cfg(test)]
mod tests;
