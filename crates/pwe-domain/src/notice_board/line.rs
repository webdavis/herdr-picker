use std::iter;

use crate::agent::age;
use crate::notice::{Level, Notice};

const MOST_LABEL_CHARACTERS: usize = 128;
const PART_BETWEEN: &str = " · ";

pub(super) fn notice_line(notice: &Notice, now: u64) -> String {
    let parts = [
        level_name(notice.level).to_string(),
        said(notice),
        ago(now.saturating_sub(notice.updated_at)),
    ];
    fit_label(&parts.join(PART_BETWEEN))
}

pub(super) fn level_name(level: Level) -> &'static str {
    match level {
        Level::NeedsYou => "needs you",
        Level::Finished => "finished",
        Level::Quiet => "quiet",
    }
}

pub(super) fn said(notice: &Notice) -> String {
    let source_and_text = format!("{}: {}", notice.source, notice.text);
    let said = match &notice.tab_label {
        Some(tab) => format!("{source_and_text}, in tab {tab}"),
        None => source_and_text,
    };
    fit_label(&said)
}

pub(super) fn group_line(count: usize, kind: &str, level: Level) -> String {
    let state = match level {
        Level::NeedsYou => "waiting",
        Level::Finished | Level::Quiet => "finished",
    };
    fit_label(&format!("{count} {} {state}", counted(count, kind)))
}

pub(super) fn reminder_line(waiting: &[&Notice]) -> Option<String> {
    let first = waiting.first()?;
    let count = waiting.len();
    let line = if waiting.iter().all(|notice| notice.kind == first.kind) {
        format!("{count} {} waiting", counted(count, &first.kind))
    } else {
        format!("{count} waiting")
    };
    Some(fit_label(&line))
}

fn ago(seconds: u64) -> String {
    if seconds == 0 {
        "now".to_string()
    } else {
        format!("{} ago", age(seconds))
    }
}

fn counted(count: usize, kind: &str) -> String {
    if count == 1 || kind.ends_with('s') {
        kind.to_string()
    } else {
        format!("{kind}s")
    }
}

pub fn fit_label(line: &str) -> String {
    let printable: Vec<char> = line.chars().filter(|glyph| !glyph.is_control()).collect();
    if printable.len() <= MOST_LABEL_CHARACTERS {
        return printable.into_iter().collect();
    }
    printable[..MOST_LABEL_CHARACTERS - 1]
        .iter()
        .copied()
        .chain(iter::once('…'))
        .collect()
}
