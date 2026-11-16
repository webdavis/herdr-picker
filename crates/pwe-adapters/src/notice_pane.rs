use std::io::{self, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseEventKind, poll, read,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use pwe_domain::{Level, NoticeEvent, NoticeStep, Theme, notice_step};

use crate::screen::palette;

const CLEAR_THE_PANE: &str = "\x1b[2J\x1b[H";

struct NoticeTerminal;

impl NoticeTerminal {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), Hide, EnableMouseCapture)?;
        Ok(Self)
    }
}

impl Drop for NoticeTerminal {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture, Show);
        let _ = disable_raw_mode();
    }
}

pub fn show_notice_pane(
    level: Level,
    theme: Theme,
    until: u64,
    now: impl Fn() -> u64,
) -> io::Result<()> {
    let _terminal = NoticeTerminal::enter()?;
    let mut event = NoticeEvent::Resized;
    loop {
        event = match notice_step(until, event) {
            NoticeStep::End => return Ok(()),
            NoticeStep::Paint => {
                paint(level, theme)?;
                NoticeEvent::TimeIs(now())
            }
            NoticeStep::Wait { seconds } if poll(Duration::from_secs(seconds))? => {
                notice_event(read()?).unwrap_or_else(|| NoticeEvent::TimeIs(now()))
            }
            NoticeStep::Wait { .. } => NoticeEvent::TimeIs(now()),
        };
    }
}

fn notice_event(event: Event) -> Option<NoticeEvent> {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Release => Some(NoticeEvent::KeyOrClick),
        Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Down(_)) => {
            Some(NoticeEvent::KeyOrClick)
        }
        Event::Resize(..) => Some(NoticeEvent::Resized),
        _ => None,
    }
}

fn paint(level: Level, theme: Theme) -> io::Result<()> {
    let colours = palette(theme);
    let background = match level {
        Level::NeedsYou => colours.needs_you,
        Level::Finished => colours.finished,
        Level::Quiet => colours.base,
    };
    let mut out = io::stdout().lock();
    write!(out, "{background}{CLEAR_THE_PANE}")?;
    out.flush()
}
