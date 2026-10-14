#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub columns: usize,
    pub rows: usize,
}

const SHARE_PERCENT: usize = 40;
const SMALLEST: Size = Size {
    columns: 60,
    rows: 14,
};
const LARGEST: Size = Size {
    columns: 100,
    rows: 30,
};
const BORDER_COLUMNS: usize = 3;

pub fn popup_size(window: Size, content_columns: usize) -> Size {
    Size {
        columns: share(window.columns)
            .clamp(SMALLEST.columns, LARGEST.columns)
            .max(content_columns + BORDER_COLUMNS)
            .min(window.columns),
        rows: share(window.rows)
            .clamp(SMALLEST.rows, LARGEST.rows)
            .min(window.rows),
    }
}

fn share(cells: usize) -> usize {
    cells * SHARE_PERCENT / 100
}

#[cfg(test)]
mod tests;
