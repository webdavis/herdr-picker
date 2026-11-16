use std::cell::RefCell;
use std::convert::Infallible;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use pwe_adapters::finder::{FinderTerminal, TerminalEvent, TerminalFinder};
use pwe_adapters::{PickerRunError, read_config};
use pwe_application::{StepFinder, StepOutcome, StepSetup};
use pwe_domain::finder::{Finder, Input, RowMatcher};
use pwe_domain::{Action, Engine, PickerCall, Theme, Trail, View, ViewState};

use super::{EngineFinder, engine_of};
use crate::picker_step::Run;

struct Asked<'a> {
    name: &'static str,
    log: &'a RefCell<Vec<&'static str>>,
}

impl StepFinder for Asked<'_> {
    type Error = Infallible;

    fn find(&self, _: StepSetup) -> Result<Option<StepOutcome>, Infallible> {
        self.log.borrow_mut().push(self.name);
        Ok(None)
    }
}

fn setup(live: bool) -> StepSetup {
    StepSetup {
        rows: Command::new("true"),
        query: String::new(),
        header: String::new(),
        view: ViewState {
            view: View::Streamed { group: None },
            titles: Vec::new(),
            description: String::new(),
            source: None,
            sources: Vec::new(),
        },
        live,
    }
}

fn engines_asked(engine: Engine, live: bool) -> Vec<&'static str> {
    let log = RefCell::new(Vec::new());
    let finder = EngineFinder {
        engine,
        fzf: Asked {
            name: "fzf",
            log: &log,
        },
        pwe: Asked {
            name: "pwe",
            log: &log,
        },
    };
    let _ = finder.find(setup(live));
    log.into_inner()
}

#[test]
fn a_live_step_stays_on_fzf() {
    assert_eq!(engines_asked(Engine::Pwe, true), ["fzf"]);
    assert_eq!(engines_asked(Engine::Pwe, false), ["pwe"]);
}

#[test]
fn a_picker_without_an_engine_runs_its_steps_in_fzf() {
    assert_eq!(engines_asked(Engine::Fzf, false), ["fzf"]);
}

struct FzfIsNeverAsked;

impl StepFinder for FzfIsNeverAsked {
    type Error = PickerRunError;

    fn find(&self, _: StepSetup) -> Result<Option<StepOutcome>, PickerRunError> {
        panic!("the pwe engine asked fzf for a step")
    }
}

struct DownThenEnter {
    listed: Vec<String>,
    keys: Vec<&'static str>,
}

impl DownThenEnter {
    fn open() -> io::Result<Self> {
        Ok(Self {
            listed: Vec::new(),
            keys: vec!["enter", "down"],
        })
    }
}

impl FinderTerminal for DownThenEnter {
    fn draw<M: RowMatcher>(&mut self, finder: &Finder<M>, _: Theme) -> io::Result<()> {
        self.listed = finder
            .listed()
            .iter()
            .map(|label| label.to_string())
            .collect();
        Ok(())
    }

    fn size(&mut self) -> io::Result<Input> {
        Ok(Input::Resized {
            list_height: 10,
            preview_width: 30,
        })
    }

    fn next_event(&mut self, wait: Duration) -> io::Result<Option<TerminalEvent>> {
        if self.listed.len() < 3 {
            std::thread::sleep(wait);
            return Ok(None);
        }
        let key = self.keys.pop().expect("the recorded keys ran out");
        Ok(Some(TerminalEvent::Key(key.to_string())))
    }
}

struct Sandbox(PathBuf);

impl Sandbox {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("pwe-engine-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("empty-path")).expect("a sandbox");
        Self(directory)
    }

    fn rows_script(&self) -> PathBuf {
        let note = self.0.join("note.txt");
        fs::write(&note, "a note\n").expect("a previewed file");
        let rows = ["one", "two", "three"].map(|label| {
            format!(
                r#"echo '{{"label":"{label}","enter":{{"insert":"{label}"}},"preview":{{"path":"{}"}}}}'"#,
                note.display()
            )
        });
        let script = self.0.join("files.sh");
        fs::write(&script, format!("#!/bin/sh\n{}\n", rows.join("\n"))).expect("a rows script");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).expect("executable");
        script
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_files_picker_runs_in_the_pwe_engine_with_an_empty_path() {
    let sandbox = Sandbox::new();
    let config_file = sandbox.0.join("config.toml");
    let body = format!(
        "[pickers.files]\nengine = \"pwe\"\ncommand = [\"{}\"]\n",
        sandbox.rows_script().display()
    );
    fs::write(&config_file, body).expect("a config");
    let config = read_config(&config_file).expect("a config");
    let finder = EngineFinder {
        engine: engine_of("files", &config),
        fzf: FzfIsNeverAsked,
        pwe: TerminalFinder {
            cache: &sandbox.0.join("rows.jsonl"),
            theme: Theme::default(),
            quickfix: &sandbox.0.join("quickfix.json"),
            path_variable: Some(OsString::from(sandbox.0.join("empty-path"))),
            open_terminal: DownThenEnter::open,
        },
    };
    let run = Run {
        config: &config,
        finder: &finder,
        quickfix: Path::new("/nonexistent/quickfix.json"),
    };
    let trail = Trail::new(PickerCall {
        name: "files".to_string(),
        args: Vec::new(),
        state: None,
        query: None,
    });

    let shown = run.show(&trail).expect("a finished step");

    assert_eq!(
        shown.map(|(action, _)| action),
        Some(Action::Insert("two".to_string()))
    );
}
