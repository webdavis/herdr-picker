use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use pwe_domain::test_engine::Retention;

use super::*;

static NEXT_FOLDER: AtomicU64 = AtomicU64::new(0);

struct TempState(PathBuf);

impl TempState {
    fn new() -> Self {
        let number = NEXT_FOLDER.fetch_add(1, Ordering::Relaxed);
        TempState(std::env::temp_dir().join(format!("pwe-runs-{}-{number}", std::process::id())))
    }

    fn runs(&self) -> PathBuf {
        run_folder(&self.0, Path::new("/home/someone/project"))
    }
}

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn each_event_is_on_disk_before_the_next_arrives() {
    let state = TempState::new();
    let mut file = RunFile::create(&state.runs(), "20261116T081500Z-3f9a").unwrap();
    let on_disk = || fs::read_to_string(state.runs().join("20261116T081500Z-3f9a.jsonl")).unwrap();

    file.append(r#"{"event":"run-started"}"#).unwrap();
    let after_first = on_disk();
    file.append(r#"{"event":"run-finished"}"#).unwrap();

    assert_eq!(after_first, "{\"event\":\"run-started\"}\n");
    assert_eq!(
        on_disk(),
        "{\"event\":\"run-started\"}\n{\"event\":\"run-finished\"}\n"
    );
}

#[test]
fn a_run_file_is_private_to_its_owner() {
    let state = TempState::new();
    RunFile::create(&state.runs(), "20261116T081500Z-3f9a").unwrap();

    assert_eq!(
        (
            mode(&state.runs()),
            mode(&state.runs().join("20261116T081500Z-3f9a.jsonl"))
        ),
        (0o700, 0o600)
    );
}

#[test]
fn each_worktree_keeps_its_runs_under_its_own_folder_path() {
    assert_eq!(
        run_folder(Path::new("/state/pwe"), Path::new("/home/someone/project")),
        PathBuf::from("/state/pwe/tests/%home%someone%project")
    );
}

#[test]
fn runs_past_the_retention_are_dropped_oldest_first() {
    let state = TempState::new();
    for run_id in [
        "20261114T000000Z-0001",
        "20261115T000000Z-0002",
        "20261116T000000Z-0003",
    ] {
        RunFile::create(&state.runs(), run_id).unwrap();
    }
    fs::write(state.runs().join("notes.txt"), "kept").unwrap();

    drop_old_runs(
        &state.runs(),
        &Retention {
            keep_runs: 2,
            keep_bytes: 1024,
        },
    )
    .unwrap();

    let mut left: Vec<String> = fs::read_dir(state.runs())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(
        left,
        [
            "20261115T000000Z-0002.jsonl",
            "20261116T000000Z-0003.jsonl",
            "notes.txt"
        ]
    );
}

#[test]
fn the_run_written_last_is_listed_first() {
    let state = TempState::new();
    let mut later = RunFile::create(&state.runs(), "20261116T000000Z-ffff").unwrap();
    let mut earlier = RunFile::create(&state.runs(), "20261116T000000Z-0000").unwrap();
    later.append("first").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    earlier.append("second").unwrap();

    assert_eq!(
        stored_run_ids_newest_first(&state.runs()),
        ["20261116T000000Z-0000", "20261116T000000Z-ffff"]
    );
}

#[test]
fn only_a_stored_run_id_is_read() {
    let state = TempState::new();
    let mut file = RunFile::create(&state.runs(), "20261116T000000Z-0001").unwrap();
    file.append("kept").unwrap();
    fs::write(state.0.join("outside.jsonl"), "secret").unwrap();

    assert_eq!(
        (
            read_stored_run(&state.runs(), "20261116T000000Z-0001"),
            read_stored_run(&state.runs(), "../../outside"),
        ),
        (Some("kept\n".to_string()), None)
    );
}
