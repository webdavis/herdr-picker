use std::io::{self, Write};

use crossterm::terminal::size;
use herdr_workspace_jump_application::Screen;
use herdr_workspace_jump_domain::{Frame, Legend};

mod view;

const HIDE_CURSOR_AND_CLEAR: &str = "\x1b[?25l\x1b[H\x1b[2J";
const SHOW_CURSOR: &str = "\x1b[?25h";
const UNKNOWN_SIZE: (usize, usize) = (0, 0);

pub struct TerminalScreen<W: Write> {
    out: W,
    size: fn() -> (usize, usize),
    legend: Legend,
}

impl TerminalScreen<io::Stdout> {
    pub fn stdout(legend: Legend) -> Self {
        Self::new(io::stdout(), terminal_size, legend)
    }
}

impl<W: Write> TerminalScreen<W> {
    pub fn new(out: W, size: fn() -> (usize, usize), legend: Legend) -> Self {
        Self { out, size, legend }
    }
}

impl<W: Write> Screen for TerminalScreen<W> {
    fn show(&mut self, frame: &Frame<'_>) -> io::Result<()> {
        let (width, height) = (self.size)();
        let drawn = view::render(frame, self.legend, width, height);
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
