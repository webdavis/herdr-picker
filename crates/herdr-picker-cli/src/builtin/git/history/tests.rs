use herdr_picker_domain::{Action, Key, PickerCall, PickerRow, Preview};

use crate::builtin::Asked;
use crate::builtin::git::fixture::{GitRepo, labels, read};
use crate::builtin::tests::{asked, printed};

const YELLOW: &str = "\x1b[38;2;249;226;175m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

fn two_commits(name: &str) -> GitRepo {
    let repo = GitRepo::new(name);
    repo.write("a.txt", "alpha\n");
    repo.commit("first");
    repo.write("b.txt", "needle\n");
    repo.commit("second");
    repo
}

fn history(repo: &GitRepo, source: &str, state: Option<&str>, query: &str) -> Vec<PickerRow> {
    let asking = Asked {
        source: Some(source),
        state,
        query,
    };
    read(&asked("history", &[], &asking, &repo.places()).expect("rows")).1
}

fn next(name: &str, args: &[&str]) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state: None,
        query: None,
    })
}

#[test]
fn commits_list_newest_first_with_their_short_hash_subject_and_age() {
    let repo = two_commits("history-commits");
    let (second, first) = (
        repo.git(&["rev-parse", "HEAD"]),
        repo.git(&["rev-parse", "HEAD~"]),
    );
    let age = repo.git(&["log", "-1", "--format=%cr, %an"]);

    let rows = history(&repo, "commits", None, "");

    assert_eq!(
        labels(&rows),
        vec![
            format!("{YELLOW}{}{RESET} second  {DIM}{age}{RESET}", &second[..7]),
            format!("{YELLOW}{}{RESET} first  {DIM}{age}{RESET}", &first[..7]),
        ]
    );
    let row = &rows[0];
    assert_eq!(row.enter, Action::Insert(second.clone()));
    assert_eq!(
        row.action(&key("ctrl-enter")),
        Some(&next("checkout", &["commit", &second]))
    );
    assert_eq!(
        row.action(&key("ctrl-t")),
        Some(&next("commit-files", &[&second]))
    );
    assert_eq!(
        row.action(&key("alt-s")),
        Some(&Action::State("{\"sort\":true}".to_string()))
    );
    let root = repo.0.display();
    assert_eq!(
        row.action(&key("alt-i")),
        Some(&Action::Run(format!("git -C {root} show {second}")))
    );
    let Some(Preview::Command(preview)) = &row.preview else {
        panic!("a command preview");
    };
    assert_eq!(preview[preview.len() - 1], second);
}

#[test]
fn oldest_first_lists_them_the_other_way_and_alt_s_turns_it_back() {
    let repo = two_commits("history-sorted");

    let rows = history(&repo, "commits", Some("{\"sort\":true}"), "");

    assert!(rows[0].label.contains(" first "), "{}", rows[0].label);
    assert_eq!(
        rows[0].action(&key("alt-s")),
        Some(&Action::State("{}".to_string()))
    );
}

#[test]
fn changes_lists_the_commits_whose_changes_match_what_is_typed_and_none_before() {
    let repo = two_commits("history-changes");

    assert!(history(&repo, "changes", None, "").is_empty());
    let rows = history(&repo, "changes", None, "needle");

    assert_eq!(rows.len(), 1);
    assert!(rows[0].label.contains(" second "), "{}", rows[0].label);
}

#[test]
fn the_reflog_says_what_moved_head() {
    let repo = two_commits("history-reflog");

    let rows = history(&repo, "reflog", None, "");

    assert!(
        rows[0].label.contains(" commit: second  "),
        "{}",
        rows[0].label
    );
    assert!(
        rows[0].label.contains(&format!("{DIM}HEAD@{{")),
        "{}",
        rows[0].label
    );
    assert!(
        rows[0].label.ends_with(&format!(" ago}}{RESET}")),
        "{}",
        rows[0].label
    );
}

#[test]
fn the_files_at_a_commit_preview_their_content_and_enter_prepares_reading_one() {
    let repo = two_commits("history-files");
    let first = repo.git(&["rev-parse", "HEAD~"]);
    let blob = repo.git(&["rev-parse", "HEAD~:a.txt"]);

    let (header, rows) =
        read(&printed("commit-files", &[&first], None, &repo.places()).expect("rows"));

    assert_eq!(
        header,
        Some(format!(
            "{} first ({})",
            &first[..7],
            repo.git(&["log", "-1", "--format=%cr", &first])
        ))
    );
    assert_eq!(labels(&rows), vec!["a.txt"]);
    let read_it = format!(
        "git -C {} cat-file blob {blob} | nvim -R -",
        repo.0.display()
    );
    assert_eq!(rows[0].enter, Action::Insert(read_it.clone()));
    assert_eq!(
        rows[0].action(&key("ctrl-enter")),
        Some(&Action::Run(read_it))
    );
    let Some(Preview::Command(preview)) = &rows[0].preview else {
        panic!("a command preview");
    };
    assert_eq!(
        preview[1..],
        [
            "git-preview",
            "blob",
            &*repo.0.to_string_lossy(),
            &blob,
            "a.txt"
        ]
    );
}
