use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

use super::*;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-pane-result-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn result(&self) -> PathBuf {
        self.0.join("picker-result")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const SOON: Duration = Duration::from_millis(300);

fn finished(instruction: Option<&str>) -> PaneState {
    PaneState::Finished {
        instruction: instruction.map(str::to_string),
    }
}

fn exited_pid() -> u32 {
    let mut child = Command::new("true").spawn().expect("true starts");
    child.wait().expect("true exits");
    child.id()
}

#[test]
fn writing_a_state_replaces_the_file_whole_only_its_owner_reads_it_and_leaves_no_temporary_behind()
{
    let sandbox = Sandbox::new("write");
    let result = sandbox.result();

    write_pane_state(&result, &PaneState::Running { pid: 7 }).expect("written");
    write_pane_state(&result, &finished(Some("insert\techo alpha"))).expect("rewritten");

    assert_eq!(
        fs::read_to_string(&result).expect("a result"),
        "finished\ninsert\techo alpha"
    );
    let names: Vec<_> = fs::read_dir(&sandbox.0)
        .expect("the sandbox lists")
        .map(|entry| entry.expect("an entry").file_name())
        .collect();
    assert_eq!(names, ["picker-result"]);
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(&result)
        .expect("a result")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "only its owner reads the result");
}

#[test]
fn a_pick_hands_its_instruction_to_the_caller() {
    let sandbox = Sandbox::new("pick");
    write_pane_state(&sandbox.result(), &finished(Some("cd\t/opt/beta"))).expect("written");

    assert_eq!(
        await_pane_result(&sandbox.result(), SOON),
        Ok(Some("cd\t/opt/beta".to_string()))
    );
}

#[test]
fn escape_hands_the_caller_nothing_to_do() {
    let sandbox = Sandbox::new("escape");
    write_pane_state(&sandbox.result(), &finished(None)).expect("written");

    assert_eq!(await_pane_result(&sandbox.result(), SOON), Ok(None));
}

#[test]
fn a_failure_in_the_popup_reaches_the_caller_word_for_word() {
    let sandbox = Sandbox::new("failure");
    let failed = PaneState::Failed {
        reason: "no picker is named files".to_string(),
    };
    write_pane_state(&sandbox.result(), &failed).expect("written");

    assert_eq!(
        await_pane_result(&sandbox.result(), SOON),
        Err(PaneWaitError::Failed(
            "no picker is named files".to_string()
        ))
    );
}

#[test]
fn the_caller_waits_while_the_popup_is_still_open() {
    let sandbox = Sandbox::new("waiting");
    let result = sandbox.result();
    write_pane_state(
        &result,
        &PaneState::Running {
            pid: std::process::id(),
        },
    )
    .expect("written");
    let writer = {
        let result = result.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            write_pane_state(&result, &finished(Some("run\tdate"))).expect("rewritten");
        })
    };

    assert_eq!(
        await_pane_result(&result, SOON),
        Ok(Some("run\tdate".to_string()))
    );
    writer.join().expect("the writer finished");
}

#[test]
fn a_popup_that_went_away_without_a_result_fails_the_caller() {
    let sandbox = Sandbox::new("gone");
    write_pane_state(&sandbox.result(), &PaneState::Running { pid: exited_pid() })
        .expect("written");

    assert_eq!(
        await_pane_result(&sandbox.result(), SOON),
        Err(PaneWaitError::ClosedWithoutResult)
    );
}

#[test]
fn a_popup_that_never_starts_fails_the_caller_once_the_start_bound_passes() {
    let sandbox = Sandbox::new("never");

    assert_eq!(
        await_pane_result(&sandbox.result(), Duration::from_millis(100)),
        Err(PaneWaitError::NeverStarted)
    );
}

#[test]
fn every_wait_failure_renders_as_one_sentence() {
    assert_eq!(
        PaneWaitError::NeverStarted.to_string(),
        "the picker popup did not start"
    );
    assert_eq!(
        PaneWaitError::ClosedWithoutResult.to_string(),
        "the picker popup closed without a result"
    );
    assert_eq!(
        PaneWaitError::Failed("no picker is named files".to_string()).to_string(),
        "no picker is named files"
    );
}
