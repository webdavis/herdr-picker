mod test_project;

use std::path::Path;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, kill_process};
use serde_json::json;
use test_project::{TestProject, exit_code, lines_of};

const STARTED: &str =
    "     Running unittests src/lib.rs (target/debug/deps/probe-1)\n\nrunning 1 test\n";
const DEADLINE: Duration = Duration::from_secs(120);

fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    let started = Instant::now();
    while !done() {
        assert!(started.elapsed() < DEADLINE, "gave up waiting for {what}");
        thread::sleep(Duration::from_millis(20));
    }
}

fn interrupt(pwe: &Child) {
    let pid = i32::try_from(pwe.id())
        .ok()
        .and_then(Pid::from_raw)
        .unwrap_or_else(|| panic!("pwe has no pid"));
    kill_process(pid, Signal::INT).unwrap_or_else(|failure| panic!("{failure}"));
}

fn sleeping_cargo(name: &str) -> TestProject {
    let project = TestProject::new(name);
    let folder = project.folder.display().to_string();
    project.file("Cargo.toml", "[workspace]\n").fake_tool_ending(
        "cargo",
        STARTED,
        &format!("trap 'touch {folder}/tool-interrupted; exit 130' INT\ntouch {folder}/tool-ready\nsleep 60"),
    );
    project
}

#[test]
fn sigint_ends_with_run_finished_cancelled_and_exit_130() {
    let project = sleeping_cargo("cancel");
    let mut pwe = project.spawn(&["test", "run", "--all", "--format", "jsonl"]);
    wait_until("the tool to start", || {
        project.folder.join("tool-ready").exists()
    });

    interrupt(&pwe);
    wait_until("pwe to finish", || matches!(pwe.try_wait(), Ok(Some(_))));
    let output = pwe
        .wait_with_output()
        .unwrap_or_else(|failure| panic!("{failure}"));
    let last = lines_of(&output).last().cloned().unwrap_or_default();

    assert_eq!(exit_code(&output), Some(130), "{output:?}");
    assert_eq!(
        (&last["event"], &last["status"], &last["exit-code"]),
        (&json!("run-finished"), &json!("cancelled"), &json!(130))
    );
}

#[test]
fn sigint_reaches_the_tools_whole_process_group() {
    let project = sleeping_cargo("cancel-group");
    let mut pwe = project.spawn(&["test", "run", "--all", "--format", "jsonl"]);
    wait_until("the tool to start", || {
        project.folder.join("tool-ready").exists()
    });

    interrupt(&pwe);
    wait_until("pwe to finish", || matches!(pwe.try_wait(), Ok(Some(_))));

    assert!(Path::new(&project.folder.join("tool-interrupted")).exists());
}
