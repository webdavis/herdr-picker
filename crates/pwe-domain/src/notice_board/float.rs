use crate::notice::Level;

#[derive(Debug, Clone)]
pub(super) struct Float {
    pub(super) id: String,
    pub(super) slot: usize,
    pub(super) level: Level,
    pub(super) members: Vec<String>,
    pub(super) shown_at: u64,
    pub(super) until: u64,
}

pub(super) fn lowest_free_slot(floats: &[Float]) -> usize {
    (0..)
        .find(|slot| floats.iter().all(|float| float.slot != *slot))
        .unwrap_or_default()
}
