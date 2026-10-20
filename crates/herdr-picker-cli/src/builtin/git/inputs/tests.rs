use herdr_picker_domain::{Action, PickerRow, Preview};

use crate::builtin::Asked;
use crate::builtin::git::fixture::{GitRepo, read};
use crate::builtin::tests::asked;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn typed(
    name: &str,
    arguments: &[&str],
    query: &str,
    repo: &GitRepo,
) -> (Option<String>, Vec<PickerRow>) {
    let asking = Asked {
        query,
        ..Asked::default()
    };
    read(&asked(name, arguments, &asking, &repo.places()).expect("rows"))
}

fn confirm(question: &str, command: String) -> Action {
    Action::Confirm {
        question: question.to_string(),
        action: Box::new(Action::Run(command)),
    }
}

#[test]
fn a_stash_message_names_the_files_and_asks_before_stashing_them() {
    let repo = GitRepo::new("input-stash");
    repo.write("a.txt", "one\n");
    repo.commit("first");
    repo.write("a.txt", "two\n");
    repo.write("new.txt", "new\n");
    let root = repo.0.display();

    let (header, rows) = typed("stash-files", &["a.txt", "new.txt"], "wip", &repo);
    let (_, untitled) = typed("stash-files", &["a.txt"], "", &repo);

    assert_eq!(
        header,
        Some("Type a message for the stash, or leave it empty, then press Enter.".to_string())
    );
    assert_eq!(rows[0].label, "Stash 2 files as \"wip\"");
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text("a.txt, new.txt".to_string()))
    );
    assert_eq!(
        rows[0].enter,
        confirm(
            "Stash 2 files as \"wip\"?",
            format!("git -C {root} stash push --include-untracked -m wip -- a.txt new.txt")
        )
    );
    assert_eq!(
        untitled[0].enter,
        confirm(
            "Stash 1 file?",
            format!("git -C {root} stash push -- a.txt")
        )
    );
}

#[test]
fn a_new_branch_name_asks_before_renaming_and_the_old_one_asks_for_another() {
    let repo = GitRepo::new("input-rename");
    repo.write("a.txt", "one\n");
    repo.commit("first");
    let root = repo.0.display();

    let (header, rows) = typed("rename-branch", &["main"], "trunk", &repo);
    let (_, unchanged) = typed("rename-branch", &["main"], "main", &repo);

    assert_eq!(
        header,
        Some("Type the branch's new name, then press Enter to review the change.".to_string())
    );
    assert_eq!(rows[0].label, "Rename main to trunk");
    assert_eq!(
        rows[0].enter,
        confirm(
            "Rename main to trunk?",
            format!("git -C {root} branch -m main trunk")
        )
    );
    assert_eq!(
        unchanged[0].label,
        format!("{DIM}Type a new name for the branch.{RESET}")
    );
    assert_eq!(unchanged[0].enter, Action::Nothing);
}

#[test]
fn a_new_setting_value_asks_before_writing_it() {
    let repo = GitRepo::new("input-setting");
    let file = repo.path("config");

    let (header, rows) = typed(
        "change-setting",
        &[&file, "pull.rebase", "global", "false"],
        "true",
        &repo,
    );

    assert_eq!(
        header,
        Some("Type the new value, then press Enter to review the change.".to_string())
    );
    assert_eq!(rows[0].label, "Set pull.rebase to true");
    assert_eq!(
        rows[0].enter,
        confirm(
            "Set pull.rebase in global from false to true?",
            format!("git config --file {file} pull.rebase true")
        )
    );
}
