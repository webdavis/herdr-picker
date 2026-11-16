use std::fs::{File, OpenOptions};
use std::io;
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::event::{
    self, Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use pwe_domain::Theme;
use pwe_domain::finder::{Finder, Input, RowMatcher};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;

use super::{FinderTerminal, TerminalEvent, draw, key_name, resized};

const TERMINAL: &str = "/dev/tty";

pub struct TtyTerminal {
    screen: Terminal<CrosstermBackend<File>>,
    _restore_on_every_exit: RestoreTerminal,
}

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        if let Ok(mut terminal) = open_terminal() {
            let _ = execute!(
                terminal,
                PopKeyboardEnhancementFlags,
                LeaveAlternateScreen,
                Show
            );
        }
        let _ = disable_raw_mode();
    }
}

impl TtyTerminal {
    pub fn open() -> io::Result<Self> {
        let mut terminal = open_terminal()?;
        enable_raw_mode()?;
        let restore = RestoreTerminal;
        execute!(
            terminal,
            EnterAlternateScreen,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
        Ok(Self {
            screen: Terminal::new(CrosstermBackend::new(terminal))?,
            _restore_on_every_exit: restore,
        })
    }
}

fn open_terminal() -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(TERMINAL)
}

impl FinderTerminal for TtyTerminal {
    fn draw<M: RowMatcher>(&mut self, finder: &Finder<M>, theme: Theme) -> io::Result<()> {
        self.screen
            .draw(|frame| draw(frame, finder, theme))
            .map(|_| ())
    }

    fn size(&mut self) -> io::Result<Input> {
        let size = self.screen.size()?;
        Ok(resized(Rect::new(0, 0, size.width, size.height)))
    }

    fn next_event(&mut self, wait: Duration) -> io::Result<Option<TerminalEvent>> {
        if !event::poll(wait)? {
            return Ok(None);
        }
        Ok(match event::read()? {
            Event::Key(key) => key_name(key).map(TerminalEvent::Key),
            Event::Resize(..) => Some(TerminalEvent::Resized),
            _ => None,
        })
    }
}
