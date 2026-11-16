use crate::Size;

const TITLE_BAR_ROWS: f64 = 2.5;
const ROOM_BESIDE_THE_LINE: usize = 6;
const WIDEST_SHARE_OF_THE_WINDOW: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Corner {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoticePlace {
    pub bounds: [f64; 4],
    pub size: Size,
}

pub fn notice_place(window: Size, corner: Corner, slot: usize, line: &str) -> NoticePlace {
    let columns = window.columns.max(1);
    let widest = ((columns as f64 * WIDEST_SHARE_OF_THE_WINDOW) as usize).max(1);
    let cells = (line.chars().count() + ROOM_BESIDE_THE_LINE).min(widest);
    let width = cells as f64 / columns as f64;
    let height = (TITLE_BAR_ROWS / window.rows.max(1) as f64).min(1.0);
    let from_the_edge = (slot as f64 * height).min(1.0 - height);
    let left = match corner {
        Corner::TopRight | Corner::BottomRight => 1.0 - width,
        Corner::TopLeft | Corner::BottomLeft => 0.0,
    };
    let top = match corner {
        Corner::TopRight | Corner::TopLeft => from_the_edge,
        Corner::BottomRight | Corner::BottomLeft => 1.0 - height - from_the_edge,
    };
    NoticePlace {
        bounds: [left, top, width, height],
        size: Size {
            columns: cells,
            rows: 1,
        },
    }
}

#[cfg(test)]
mod tests;
