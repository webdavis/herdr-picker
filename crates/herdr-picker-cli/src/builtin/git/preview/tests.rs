use super::*;
use crate::builtin::git::fixture::GitRepo;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn plain(text: &str) -> String {
    let mut plain = String::new();
    let mut escaped = false;
    for character in text.chars() {
        match (escaped, character) {
            (_, '\x1b') => escaped = true,
            (true, 'm') => escaped = false,
            (true, _) => {}
            (false, character) => plain.push(character),
        }
    }
    plain
}

#[test]
fn a_change_shows_its_staged_then_its_unstaged_part_each_under_its_heading() {
    let repo = GitRepo::new("preview-diff");
    repo.write("a.txt", "one\n");
    repo.commit("first");
    repo.write("a.txt", "two\n");
    repo.git(&["add", "a.txt"]);
    repo.write("a.txt", "three\n");

    let shown = git_preview(&["diff", &repo.path(""), "a.txt"]).expect("a preview");

    assert!(
        shown.starts_with(&format!("{BOLD}Staged{RESET}\n")),
        "{shown}"
    );
    let shown = plain(&shown);
    let staged = shown.find("+two").expect("the staged change");
    let unstaged = shown.find("Unstaged\n").expect("the unstaged heading");
    assert!(
        staged < unstaged && shown[unstaged..].contains("+three"),
        "{shown}"
    );
}

#[test]
fn a_conflict_shows_base_ours_and_theirs_and_says_when_a_side_is_absent() {
    let repo = GitRepo::new("preview-stages");
    repo.write("a.txt", "ours\n");
    repo.commit("first");
    let blob = repo.git(&["rev-parse", "HEAD:a.txt"]);
    let absent = "0".repeat(40);

    let shown = git_preview(&["stages", &repo.path(""), "a.txt", &absent, &blob, &absent])
        .expect("a preview");

    let shown = plain(&shown);
    assert!(
        shown.starts_with("Base\n(stage absent)\n\nOurs\n"),
        "{shown}"
    );
    assert!(shown.contains("ours"), "{shown}");
    assert!(shown.ends_with("Theirs\n(stage absent)"), "{shown}");
}

#[test]
fn a_binary_blob_says_so() {
    let repo = GitRepo::new("preview-binary");
    repo.write("a.bin", "a\0b");
    repo.commit("first");
    let blob = repo.git(&["rev-parse", "HEAD:a.bin"]);

    assert_eq!(
        git_preview(&["blob", &repo.path(""), &blob, "a.bin"]).expect("a preview"),
        "[Binary file]"
    );
}
