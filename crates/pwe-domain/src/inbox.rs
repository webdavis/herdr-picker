use crate::notice::Notice;

const SECONDS_A_DAY: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxLimits {
    pub keep_days: u64,
    pub most_rows: usize,
}

impl Default for InboxLimits {
    fn default() -> Self {
        Self {
            keep_days: 30,
            most_rows: 2000,
        }
    }
}

pub fn prune_inbox(rows: Vec<Notice>, now: u64, limits: &InboxLimits) -> Vec<Notice> {
    let oldest_kept = now.saturating_sub(limits.keep_days * SECONDS_A_DAY);
    let mut kept: Vec<Notice> = rows
        .into_iter()
        .filter(|row| row.is_unread() || row.last_changed_at() >= oldest_kept)
        .collect();
    let mut read_oldest_first: Vec<(u64, String)> = kept
        .iter()
        .filter(|row| !row.is_unread())
        .map(|row| (row.last_changed_at(), row.id.clone()))
        .collect();
    read_oldest_first.sort();
    let surplus = kept.len().saturating_sub(limits.most_rows);
    let dropped: Vec<String> = read_oldest_first
        .into_iter()
        .take(surplus)
        .map(|(_, id)| id)
        .collect();
    kept.retain(|row| !dropped.contains(&row.id));
    kept
}

#[cfg(test)]
mod tests;
