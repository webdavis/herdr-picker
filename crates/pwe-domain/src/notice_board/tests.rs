use super::*;
use crate::notice::{Level, Pane};

mod grouping;
mod showing;
mod updating;

fn pane_in(rex_session: &str, tab: &str, block: &str) -> Pane {
    Pane {
        rex_session: rex_session.to_string(),
        tab: tab.to_string(),
        block: block.to_string(),
    }
}

fn pane(block: &str) -> Pane {
    pane_in("work", "api", block)
}

fn raised(key: &str, level: Level) -> Raised {
    Raised {
        key: key.to_string(),
        source: "tests".to_string(),
        kind: key.to_string(),
        level,
        text: format!("{key} done"),
        pane: None,
        tab_label: None,
    }
}

fn raise(key: &str, level: Level) -> NoticeInput {
    NoticeInput::Raise(raised(key, level))
}

fn board_in_front() -> NoticeBoard {
    let mut board = NoticeBoard::new(NoticeSettings::default());
    board.apply(NoticeInput::OwnerMoved(RexFocus::Front(pane("view"))), 0);
    board
}

#[derive(Debug, PartialEq, Eq)]
struct Shown {
    float: String,
    slot: usize,
    level: Level,
    line: String,
    seconds: u64,
}

fn shown(effects: &[NoticeEffect]) -> Vec<Shown> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            NoticeEffect::Show {
                float,
                slot,
                level,
                line,
                seconds,
            } => Some(Shown {
                float: float.clone(),
                slot: *slot,
                level: *level,
                line: line.clone(),
                seconds: *seconds,
            }),
            _ => None,
        })
        .collect()
}

fn closed(effects: &[NoticeEffect]) -> Vec<String> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            NoticeEffect::Close { float } => Some(float.clone()),
            _ => None,
        })
        .collect()
}

fn saved_rows(effects: &[NoticeEffect]) -> Vec<crate::notice::Notice> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            NoticeEffect::SaveRow(row) => Some((**row).clone()),
            _ => None,
        })
        .collect()
}
