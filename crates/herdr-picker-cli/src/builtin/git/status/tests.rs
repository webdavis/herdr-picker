use herdr_picker_domain::{Action, Key, Location, PickerCall, Preview};

use crate::builtin::git::fixture::{GitRepo, labels, read};
use crate::builtin::tests::printed;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const YELLOW: &str = "\x1b[38;2;249;226;175m";

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

// a.txt staged and changed again, c.txt new, d.log ignored, and b.txt's change in a stash.
fn worked_on(name: &str) -> GitRepo {
    let repo = GitRepo::new(name);
    repo.write(".gitignore", "*.log\n");
    repo.write("a.txt", "one\n");
    repo.write("b.txt", "one\n");
    repo.commit("first");
    repo.write("b.txt", "two\n");
    repo.git(&["stash", "push", "-q", "-m", "wip b"]);
    repo.write("a.txt", "two\n");
    repo.git(&["add", "a.txt"]);
    repo.write("a.txt", "three\n");
    repo.write("c.txt", "new\n");
    repo.write("d.log", "noise\n");
    repo
}

fn status(repo: &GitRepo, source: &str) -> (Option<String>, Vec<herdr_picker_domain::PickerRow>) {
    read(&printed("status", &[], Some(source), &repo.places()).expect("rows"))
}

#[test]
fn all_lists_each_file_once_under_its_first_state_then_the_stashes_then_the_quiet_files() {
    let repo = worked_on("status-all");

    let (header, rows) = status(&repo, "all");

    assert_eq!(
        header,
        Some("main | 1 staged, 1 unstaged, 1 untracked".to_string())
    );
    assert_eq!(
        labels(&rows),
        vec![
            format!("{DIM}staged   {RESET}  a.txt"),
            format!("{DIM}untracked{RESET}  c.txt"),
            format!("{YELLOW}stash@{{0}}{RESET} On main: wip b"),
            format!("{DIM}tracked  {RESET}  .gitignore"),
            format!("{DIM}tracked  {RESET}  b.txt"),
        ]
    );
}

#[test]
fn a_changed_file_previews_its_diff_inserts_its_path_and_can_be_edited_or_stashed() {
    let repo = worked_on("status-changed");

    let (_, rows) = status(&repo, "unstaged");

    assert_eq!(labels(&rows), vec!["a.txt"]);
    let row = &rows[0];
    assert_eq!(row.enter, Action::Insert("a.txt".to_string()));
    let Some(Preview::Command(preview)) = &row.preview else {
        panic!("a command preview: {:?}", row.preview);
    };
    assert_eq!(
        preview[1..],
        ["git-preview", "diff", &*repo.0.to_string_lossy(), "a.txt"]
    );
    assert_eq!(
        row.action(&key("ctrl-enter")),
        Some(&Action::Edit(vec![Location {
            path: repo.path("a.txt"),
            line: None
        }]))
    );
    assert_eq!(
        row.action(&key("ctrl-z")),
        Some(&Action::Next(PickerCall {
            name: "stash-files".to_string(),
            args: vec!["a.txt".to_string()],
            state: None,
            query: None,
        }))
    );
}

#[test]
fn a_stash_previews_its_patch_and_ctrl_enter_asks_before_applying_it() {
    let repo = worked_on("status-stash");

    let (_, rows) = status(&repo, "stashes");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].enter, Action::Insert("stash@{0}".to_string()));
    assert_eq!(
        rows[0].action(&key("ctrl-enter")),
        Some(&Action::Confirm {
            question: "Apply stash@{0} (On main: wip b)?".to_string(),
            action: Box::new(Action::Run(format!(
                "git -C {} stash apply 'stash@{{0}}'",
                repo.0.display()
            ))),
        })
    );
    assert_eq!(rows[0].action(&key("ctrl-z")), None);
}

#[test]
fn ignored_files_are_read_only_from_their_source_and_open_the_rule_that_ignores_them() {
    let repo = worked_on("status-ignored");

    let (_, all) = status(&repo, "all");
    let (_, rows) = status(&repo, "ignored");

    assert!(!labels(&all).iter().any(|label| label.contains("d.log")));
    assert_eq!(labels(&rows), vec![format!("d.log  {DIM}*.log{RESET}")]);
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "Rule: *.log\nSource: .gitignore:1".to_string()
        ))
    );
    assert_eq!(
        rows[0].action(&key("ctrl-enter")),
        Some(&Action::Edit(vec![Location {
            path: repo.path(".gitignore"),
            line: Some(1)
        }]))
    );
}

#[test]
fn a_conflict_previews_its_three_sides_and_the_header_counts_it() {
    let repo = GitRepo::new("status-conflict");
    repo.write("a.txt", "base\n");
    repo.commit("base");
    repo.git(&["checkout", "-q", "-b", "other"]);
    repo.write("a.txt", "theirs\n");
    repo.commit("theirs");
    repo.git(&["checkout", "-q", "main"]);
    repo.write("a.txt", "ours\n");
    repo.commit("ours");
    let _ = std::process::Command::new("git")
        .args([
            "-C",
            &*repo.0.to_string_lossy(),
            "-c",
            "user.name=A",
            "-c",
            "user.email=a@b",
            "merge",
            "-q",
            "other",
        ])
        .output();

    let (header, rows) = status(&repo, "conflicts");

    assert_eq!(header, Some("main | 1 conflicts".to_string()));
    assert_eq!(labels(&rows), vec!["a.txt"]);
    let Some(Preview::Command(preview)) = &rows[0].preview else {
        panic!("a command preview");
    };
    assert_eq!(
        preview[2..5],
        ["stages", &*repo.0.to_string_lossy(), "a.txt"]
    );
    assert_eq!(preview.len(), 8);
}

#[test]
fn outside_a_repository_the_picker_says_it_needs_one() {
    let places = crate::builtin::Places {
        cwd: std::env::temp_dir(),
        home: None,
        fd: None,
        bat: None,
        bookmarks: std::env::temp_dir().join("bookmarks"),
        quickfix: std::env::temp_dir().join("quickfix.json"),
        captures: std::env::temp_dir().join("captures"),
    };

    let refusal =
        printed("status", &[], Some("all"), &places).map_err(|failure| format!("{failure:?}"));

    assert_eq!(
        refusal,
        Err("Failed(\"This picker needs a Git repository.\")".to_string())
    );
}
