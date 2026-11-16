mod test_project;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use test_project::{TestProject, json_of, stdout_of};

const PASSING: &str = "     Running unittests src/lib.rs (target/debug/deps/probe-1)\n\nrunning 1 test\ntest tests::passes ... ok\n";

fn runs_folder(project: &TestProject) -> PathBuf {
    let repo_key = project.project().to_string_lossy().replace('/', "%");
    project.state().join("pwe/tests").join(repo_key)
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path)
        .unwrap_or_else(|failure| panic!("{}: {failure}", path.display()))
        .permissions()
        .mode()
        & 0o777
}

#[test]
fn a_run_is_kept_as_a_private_file_holding_the_events_it_streamed() {
    let project = TestProject::new("history-kept");
    project
        .file("Cargo.toml", "[workspace]\n")
        .fake_tool("cargo", PASSING, 0);

    let streamed = stdout_of(&project.pwe(&["test", "run", "--all", "--format", "jsonl"]));
    let run_id = json_of(streamed.lines().next().unwrap_or_default())["run-id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let kept = runs_folder(&project).join(format!("{run_id}.jsonl"));

    assert_eq!(fs::read_to_string(&kept).unwrap_or_default(), streamed);
    assert_eq!((mode(&runs_folder(&project)), mode(&kept)), (0o700, 0o600));
}

#[test]
fn a_run_shown_as_text_is_kept_as_event_lines() {
    let project = TestProject::new("history-text");
    project
        .file("Cargo.toml", "[workspace]\n")
        .fake_tool("cargo", PASSING, 0);

    project.pwe(&["test", "run", "--all", "--format", "text"]);
    let kept: Vec<PathBuf> = fs::read_dir(runs_folder(&project))
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();

    assert_eq!(kept.len(), 1, "{kept:?}");
    let last = fs::read_to_string(&kept[0]).unwrap_or_default();
    assert!(
        last.lines()
            .last()
            .is_some_and(|line| json_of(line)["event"] == "run-finished"),
        "{last}"
    );
}
