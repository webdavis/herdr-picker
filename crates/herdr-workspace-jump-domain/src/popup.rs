#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub columns: usize,
    pub rows: usize,
}

const WIDTH_PERCENT: usize = 74;
const HEIGHT_PERCENT: usize = 62;
const BORDER_COLUMNS: usize = 3;

pub fn popup_size(window: Size, content_columns: usize) -> Size {
    Size {
        columns: share(window.columns, WIDTH_PERCENT)
            .max(content_columns + BORDER_COLUMNS)
            .min(window.columns),
        rows: share(window.rows, HEIGHT_PERCENT),
    }
}

fn share(cells: usize, percent: usize) -> usize {
    cells * percent / 100
}

#[cfg(test)]
mod tests;
