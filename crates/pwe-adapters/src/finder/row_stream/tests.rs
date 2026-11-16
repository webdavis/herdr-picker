use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use pwe_domain::finder::{Finder, Input, step};
use pwe_domain::{Fuzzy, Theme};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::RowStream;
use crate::finder::draw;
use crate::pane_result::alive;

const PATIENCE: Duration = Duration::from_secs(10);

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("pwe-row-stream-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn rows_command(&self, body: &str) -> Command {
        let path = self.0.join("rows.sh");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("a rows script");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable");
        Command::new(path)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn row_line(label: &str) -> String {
    format!(r#"echo '{{"label":"{label}","enter":"none"}}'"#)
}

fn until<T>(mut ready: impl FnMut() -> Option<T>) -> T {
    let started = Instant::now();
    loop {
        if let Some(found) = ready() {
            return found;
        }
        assert!(started.elapsed() < PATIENCE, "gave up waiting");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn screen(finder: &Finder<Fuzzy>) -> String {
    let mut terminal = Terminal::new(TestBackend::new(40, 6)).expect("a test terminal");
    terminal
        .draw(|frame| draw(frame, finder, Theme::default()))
        .expect("a drawn frame");
    let buffer = terminal.backend().buffer();
    buffer.content().iter().map(|cell| cell.symbol()).collect()
}

fn wait_for_file(path: &Path) -> String {
    format!("while [ ! -e '{}' ]; do sleep 0.01; done", path.display())
}

#[test]
fn rows_arriving_after_the_first_frame_join_the_list() {
    let sandbox = Sandbox::new("join");
    let go_on = sandbox.file("go-on");
    let body = [row_line("first"), wait_for_file(&go_on), row_line("second")].join("\n");
    let (inputs, arrivals) = mpsc::channel();
    let _stream = RowStream::start(
        &mut sandbox.rows_command(&body),
        &sandbox.file("rows.jsonl"),
        inputs,
    )
    .expect("a started stream");

    let mut finder = Finder::new(Fuzzy);
    let first = until(|| arrivals.try_recv().ok());
    finder = step(finder, first).0;
    let first_frame = screen(&finder);
    fs::write(&go_on, "").expect("the signal to print more");
    loop {
        let input = until(|| arrivals.try_recv().ok());
        let ended = input == Input::RowsEnded;
        finder = step(finder, input).0;
        if ended {
            break;
        }
    }

    assert!(first_frame.contains("first") && !first_frame.contains("second"));
    assert!(screen(&finder).contains("second"));
    assert_eq!(finder.listed(), ["first", "second"]);
}

#[test]
fn closing_stops_the_row_command() {
    let sandbox = Sandbox::new("close");
    let pid_file = sandbox.file("pid");
    let body = format!(
        "{}\necho $$ > '{}'\nexec sleep 30",
        row_line("only"),
        pid_file.display()
    );
    let (inputs, arrivals) = mpsc::channel();
    let stream = RowStream::start(
        &mut sandbox.rows_command(&body),
        &sandbox.file("rows.jsonl"),
        inputs,
    )
    .expect("a started stream");
    let pid: u32 = until(|| fs::read_to_string(&pid_file).ok()?.trim().parse().ok());
    let started = Instant::now();

    drop(stream);

    assert!(
        started.elapsed() < PATIENCE,
        "closing waited for the command"
    );
    assert!(!alive(pid), "the row command still runs");
    assert!(matches!(
        arrivals.try_iter().next(),
        Some(Input::RowsArrived(_))
    ));
}
