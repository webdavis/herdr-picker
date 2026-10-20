use super::porcelain::{Porcelain, STATES};

// What git status says first, such as main → origin/main ↑1 | 2 staged, 1 untracked.
pub(super) fn summary(status: &Porcelain) -> String {
    let mut place = match status.head.as_str() {
        "(detached)" => "detached HEAD".to_string(),
        head => head.to_string(),
    };
    if !status.upstream.is_empty() {
        place += &format!(" → {}", status.upstream);
        for (arrow, count) in [("↑", &status.ahead), ("↓", &status.behind)] {
            if !count.is_empty() && count != "0" {
                place += &format!(" {arrow}{count}");
            }
        }
    }
    let counts: Vec<String> = STATES[..4]
        .iter()
        .map(|state| {
            (
                state,
                status
                    .files
                    .iter()
                    .filter(|file| file.states.contains(state))
                    .count(),
            )
        })
        .filter(|(_, count)| *count > 0)
        .map(|(state, count)| format!("{count} {state}"))
        .collect();
    match counts.is_empty() {
        true => format!("{place} | nothing to commit"),
        false => format!("{place} | {}", counts.join(", ")),
    }
}
