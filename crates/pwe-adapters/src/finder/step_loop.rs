use std::io;
use std::ops::ControlFlow;
use std::path::Path;
use std::time::{Duration, Instant};

use pwe_application::StepOutcome;
use pwe_domain::Theme;
use pwe_domain::finder::{Effect, Finder, Found, Input, RowMatcher, step};

use super::{Arrivals, PreviewWorker};
use crate::clipboard::copy_to_clipboard;
use crate::picker_run::PickerRunError;
use crate::quickfix::save_quickfix;

const WAIT_FOR_A_KEY: Duration = Duration::from_millis(20);

pub enum TerminalEvent {
    Key(String),
    Resized,
}

pub trait FinderTerminal {
    fn draw<M: RowMatcher>(&mut self, finder: &Finder<M>, theme: Theme) -> io::Result<()>;
    fn size(&mut self) -> io::Result<Input>;
    fn next_event(&mut self, wait: Duration) -> io::Result<Option<TerminalEvent>>;
}

pub struct StepRun<'a, M> {
    pub finder: Finder<M>,
    pub arrivals: Arrivals,
    pub previews: &'a PreviewWorker,
    pub theme: Theme,
    pub quickfix: &'a Path,
}

pub fn run_step<M: RowMatcher, T: FinderTerminal>(
    terminal: &mut T,
    run: StepRun<M>,
) -> Result<Option<StepOutcome>, PickerRunError> {
    let StepRun {
        mut finder,
        mut arrivals,
        previews,
        theme,
        quickfix,
    } = run;
    let mut inputs = vec![terminal.size().map_err(terminal_failed)?];
    loop {
        inputs.extend(arrivals.ready(Instant::now()));
        let mut redraw = false;
        for input in inputs.drain(..) {
            match apply(finder, input, previews, quickfix)? {
                ControlFlow::Continue((next, drawn)) => (finder, redraw) = (next, redraw || drawn),
                ControlFlow::Break(outcome) => return Ok(outcome),
            }
        }
        if redraw {
            terminal.draw(&finder, theme).map_err(terminal_failed)?;
        }
        match terminal
            .next_event(WAIT_FOR_A_KEY)
            .map_err(terminal_failed)?
        {
            Some(TerminalEvent::Key(key)) => inputs.push(Input::Key(key)),
            Some(TerminalEvent::Resized) => inputs.push(terminal.size().map_err(terminal_failed)?),
            None => {}
        }
    }
}

type Stepped<M> = ControlFlow<Option<StepOutcome>, (Finder<M>, bool)>;

fn apply<M: RowMatcher>(
    finder: Finder<M>,
    input: Input,
    previews: &PreviewWorker,
    quickfix: &Path,
) -> Result<Stepped<M>, PickerRunError> {
    let (finder, effects) = step(finder, input);
    let mut redraw = false;
    for effect in effects {
        match effect {
            Effect::Draw => redraw = true,
            Effect::ShowPreview {
                row,
                preview,
                width,
            } => previews.ask(row, preview, width),
            Effect::Copy(text) => copy_to_clipboard(&text).map_err(failed("cannot copy"))?,
            Effect::SaveQuickfix(entries) => save_quickfix(quickfix, &entries)
                .map_err(failed("cannot save the quickfix list"))?,
            Effect::Close(found) => return Ok(ControlFlow::Break(Some(outcome(&finder, found)))),
            Effect::CloseWithoutFinding => return Ok(ControlFlow::Break(None)),
        }
    }
    Ok(ControlFlow::Continue((finder, redraw)))
}

fn outcome<M: RowMatcher>(finder: &Finder<M>, found: Found) -> StepOutcome {
    let rows = found
        .rows
        .iter()
        .filter_map(|&row| finder.rows().get(row).cloned())
        .collect();
    StepOutcome {
        found: Found {
            query: found.query,
            ending: found.ending,
            rows,
        },
        view_left: finder.view().clone(),
    }
}

fn failed(doing: &str) -> impl Fn(io::Error) -> PickerRunError + '_ {
    move |failure| PickerRunError::FinderFailed(format!("{doing}: {failure}"))
}

fn terminal_failed(failure: impl std::fmt::Display) -> PickerRunError {
    PickerRunError::TerminalFailed(failure.to_string())
}

#[cfg(test)]
mod tests;
