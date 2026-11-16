use std::collections::VecDeque;
use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use pwe_domain::finder::{Ending, Finder, Input, RowMatcher};
use pwe_domain::{Action, Fuzzy, Key, PickerRow, Theme};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use super::{FinderTerminal, StepRun, TerminalEvent, run_step};
use crate::finder::{Arrivals, PreviewWorker, draw, resized};

struct RecordedTerminal {
    screen: Terminal<TestBackend>,
    keys: VecDeque<&'static str>,
    shown_before_typing: &'static str,
    started: Instant,
}

impl RecordedTerminal {
    fn new(keys: &[&'static str], shown_before_typing: &'static str) -> Self {
        Self {
            screen: Terminal::new(TestBackend::new(60, 12)).expect("a test terminal"),
            keys: keys.iter().copied().collect(),
            shown_before_typing,
            started: Instant::now(),
        }
    }

    fn shows(&self, text: &str) -> bool {
        let buffer = self.screen.backend().buffer();
        let shown: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        shown.contains(text)
    }
}

impl FinderTerminal for RecordedTerminal {
    fn draw<M: RowMatcher>(&mut self, finder: &Finder<M>, theme: Theme) -> io::Result<()> {
        self.screen
            .draw(|frame| draw(frame, finder, theme))
            .map(|_| ())
            .map_err(io::Error::other)
    }

    fn size(&mut self) -> io::Result<Input> {
        Ok(resized(Rect::new(0, 0, 60, 12)))
    }

    fn next_event(&mut self, _: Duration) -> io::Result<Option<TerminalEvent>> {
        if !self.shows(self.shown_before_typing) {
            assert!(
                self.started.elapsed() < Duration::from_secs(10),
                "the screen never showed {}",
                self.shown_before_typing
            );
            return Ok(None);
        }
        let key = self.keys.pop_front().expect("the recorded keys ran out");
        Ok(Some(TerminalEvent::Key(key.to_string())))
    }
}

fn row(label: &str) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Insert(label.to_string()),
        keys: Vec::new(),
        group: None,
    }
}

fn run(
    terminal: &mut RecordedTerminal,
    rows: Vec<PickerRow>,
) -> Option<pwe_application::StepOutcome> {
    let (inputs, received) = mpsc::channel();
    let previews = PreviewWorker::start(inputs.clone(), None);
    inputs.send(Input::RowsArrived(rows)).expect("sent");
    inputs.send(Input::RowsEnded).expect("sent");
    let run = StepRun {
        finder: Finder::new(Fuzzy),
        arrivals: Arrivals::new(received),
        previews: &previews,
        theme: Theme::default(),
        quickfix: Path::new("/nonexistent/quickfix.json"),
    };
    run_step(terminal, run).expect("a finished step")
}

#[test]
fn a_recorded_key_sequence_finds_the_second_row() {
    let mut terminal = RecordedTerminal::new(&["down", "enter"], "three");

    let outcome = run(&mut terminal, vec![row("one"), row("two"), row("three")]);

    let found = outcome.expect("a found row").found;
    assert_eq!(found.ending, Ending::Key(Key::enter()));
    assert_eq!(found.rows, [row("two")]);
}

#[test]
fn escape_goes_back_with_the_view_the_user_left() {
    let mut terminal = RecordedTerminal::new(&["t", "esc"], "two");

    let outcome = run(&mut terminal, vec![row("one"), row("two")]).expect("an outcome");

    assert_eq!(outcome.found.ending, Ending::Back);
    assert_eq!(outcome.found.query, "t");
}

#[test]
fn ctrl_c_closes_without_finding() {
    let mut terminal = RecordedTerminal::new(&["ctrl-c"], "one");

    assert_eq!(run(&mut terminal, vec![row("one")]), None);
}
