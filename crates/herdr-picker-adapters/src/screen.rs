use std::io::{self, Write};

use crossterm::terminal::size;
use herdr_picker_application::Screen;
use herdr_picker_domain::Frame;

mod view;

pub use view::Layout;
pub(crate) use view::palette;

const HIDE_CURSOR_AND_CLEAR: &str = "\x1b[?25l\x1b[H\x1b[2J";
const SHOW_CURSOR: &str = "\x1b[?25h";
const UNKNOWN_SIZE: (usize, usize) = (0, 0);

pub struct TerminalScreen<W: Write> {
    out: W,
    size: fn() -> (usize, usize),
    layout: Layout,
}

impl TerminalScreen<io::Stdout> {
    pub fn stdout(layout: Layout) -> Self {
        Self::new(io::stdout(), terminal_size, layout)
    }
}

impl<W: Write> TerminalScreen<W> {
    fn new(out: W, size: fn() -> (usize, usize), layout: Layout) -> Self {
        Self { out, size, layout }
    }
}

impl<W: Write> Screen for TerminalScreen<W> {
    fn show(&mut self, frame: &Frame<'_>) -> io::Result<()> {
        let (width, height) = (self.size)();
        let drawn = view::render(frame, &self.layout, width, height);
        let prompt_cursor = if frame.search.is_some() {
            SHOW_CURSOR
        } else {
            ""
        };
        write!(self.out, "{HIDE_CURSOR_AND_CLEAR}{drawn}{prompt_cursor}")?;
        self.out.flush()
    }
}

impl<W: Write> Drop for TerminalScreen<W> {
    fn drop(&mut self) {
        let _ = write!(self.out, "{SHOW_CURSOR}");
        let _ = self.out.flush();
    }
}

fn terminal_size() -> (usize, usize) {
    size()
        .map(|(columns, rows)| (usize::from(columns), usize::from(rows)))
        .unwrap_or(UNKNOWN_SIZE)
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
