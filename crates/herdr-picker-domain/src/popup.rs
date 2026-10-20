#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub columns: usize,
    pub rows: usize,
}

struct Proportion {
    share_percent: usize,
    smallest: Size,
    largest: Size,
}

const WORKSPACE_PICKER: Proportion = Proportion {
    share_percent: 40,
    smallest: Size {
        columns: 60,
        rows: 14,
    },
    largest: Size {
        columns: 100,
        rows: 30,
    },
};
// fzf splits a preview picker in half, so it gets most of the window rather than a list-sized box.
const PREVIEW_PICKER: Proportion = Proportion {
    share_percent: 80,
    smallest: Size {
        columns: 60,
        rows: 14,
    },
    largest: Size {
        columns: 180,
        rows: 40,
    },
};
const BORDER_COLUMNS: usize = 3;

pub fn popup_size(window: Size, content_columns: usize) -> Size {
    let share = WORKSPACE_PICKER.of(window);
    Size {
        columns: share
            .columns
            .max(content_columns + BORDER_COLUMNS)
            .min(window.columns),
        rows: share.rows,
    }
}

pub fn preview_popup_size(window: Size) -> Size {
    PREVIEW_PICKER.of(window)
}

impl Proportion {
    fn of(&self, window: Size) -> Size {
        Size {
            columns: self.side(window.columns, self.smallest.columns, self.largest.columns),
            rows: self.side(window.rows, self.smallest.rows, self.largest.rows),
        }
    }

    fn side(&self, cells: usize, smallest: usize, largest: usize) -> usize {
        (cells * self.share_percent / 100)
            .clamp(smallest, largest)
            .min(cells)
    }
}

#[cfg(test)]
mod tests;
