use crate::notice::{Level, Notice};

pub(super) fn notice_line(notice: &Notice) -> String {
    notice.text.clone()
}

pub(super) fn group_line(count: usize, kind: &str, level: Level) -> String {
    let state = match level {
        Level::NeedsYou => "waiting",
        Level::Finished | Level::Quiet => "finished",
    };
    format!("{count} {} {state}", counted(count, kind))
}

fn counted(count: usize, kind: &str) -> String {
    if count == 1 || kind.ends_with('s') {
        kind.to_string()
    } else {
        format!("{kind}s")
    }
}

pub(super) fn reminder_line(waiting: &[&Notice]) -> Option<String> {
    let first = waiting.first()?;
    let count = waiting.len();
    if waiting.iter().all(|notice| notice.kind == first.kind) {
        Some(format!("{count} {} waiting", counted(count, &first.kind)))
    } else {
        Some(format!("{count} waiting"))
    }
}
