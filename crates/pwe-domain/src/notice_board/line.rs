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

pub(super) fn counted(count: usize, kind: &str) -> String {
    if count == 1 || kind.ends_with('s') {
        kind.to_string()
    } else {
        format!("{kind}s")
    }
}
