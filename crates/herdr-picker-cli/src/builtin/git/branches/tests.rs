use herdr_picker_adapters::on_path;
use herdr_picker_domain::{Action, Key, PickerCall, PickerRow};

use crate::builtin::git::fixture::{GitRepo, read};
use crate::builtin::tests::printed;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

fn next(name: &str, args: &[&str], query: Option<&str>, state: Option<&str>) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state: state.map(str::to_string),
        query: query.map(str::to_string),
    })
}

fn confirm(question: &str, command: String) -> Action {
    Action::Confirm {
        question: question.to_string(),
        action: Box::new(Action::Run(command)),
    }
}

// main checked out here, other beside it, side in its own worktree, a remote branch and a tag.
fn refs(name: &str) -> (GitRepo, String) {
    let repo = GitRepo::new(name);
    repo.write("a.txt", "one\n");
    repo.commit("first");
    repo.git(&["branch", "other"]);
    repo.git(&["update-ref", "refs/remotes/origin/topic", "main"]);
    repo.git(&["tag", "v1"]);
    let side = format!("{}-side", repo.0.display());
    let _ = std::fs::remove_dir_all(&side);
    repo.git(&["worktree", "add", "-q", "-b", "side", &side]);
    (repo, side)
}

fn branches(repo: &GitRepo) -> Vec<PickerRow> {
    read(&printed("branches", &[], Some("branches"), &repo.places()).expect("rows")).1
}

fn row_of<'a>(rows: &'a [PickerRow], enter: &str) -> &'a PickerRow {
    rows.iter()
        .find(|row| row.enter == Action::Insert(enter.to_string()))
        .unwrap_or_else(|| panic!("a row inserting {enter}"))
}

#[test]
fn local_and_remote_branches_tags_and_worktrees_are_listed_under_their_groups() {
    let (repo, side) = refs("branches-groups");

    let rows = branches(&repo);

    let groups: Vec<(&str, Action)> = rows
        .iter()
        .map(|row| (row.group.as_deref().unwrap_or_default(), row.enter.clone()))
        .collect();
    let insert = |text: &str| Action::Insert(text.to_string());
    assert_eq!(
        groups,
        vec![
            ("local branches", insert("main")),
            ("local branches", insert("other")),
            ("local branches", insert("side")),
            ("remote branches", insert("origin/topic")),
            ("tags", insert("v1")),
            ("worktrees", insert(&side)),
        ]
    );
    assert!(
        row_of(&rows, "side")
            .label
            .ends_with(&format!("{DIM}in {side}{RESET}"))
    );
    let _ = std::fs::remove_dir_all(&side);
}

#[test]
fn a_local_branch_checks_out_opens_as_a_worktree_renames_and_reaches_the_repository_keys() {
    let (repo, side) = refs("branches-local");
    let root = repo.0.display().to_string();

    let rows = branches(&repo);
    let other = row_of(&rows, "other");

    assert_eq!(
        other.action(&key("ctrl-enter")),
        Some(&next("checkout", &["local", "other"], None, None))
    );
    assert_eq!(
        other.action(&key("ctrl-t")),
        Some(&confirm(
            "Open other in its own worktree?",
            format!("wt -C {root} switch other")
        ))
    );
    assert_eq!(other.action(&key("ctrl-x")), None);
    assert_eq!(
        other.action(&key("ctrl-e")),
        Some(&next("rename-branch", &["other"], Some("other"), None))
    );
    assert_eq!(
        other.action(&key("alt-d")),
        Some(&next("prune", &[&root], None, None))
    );
    assert_eq!(
        other.action(&key("ctrl-l")),
        Some(&next("pull", &[], None, None))
    );
    assert_eq!(
        other.action(&key("ctrl-f")),
        Some(&Action::Run(format!("git -C {root} fetch --all --prune")))
    );
    assert_eq!(
        other.action(&key("alt-i")),
        Some(&Action::Run(format!("git -C {root} show refs/heads/other")))
    );
    let held = row_of(&rows, "side");
    assert_eq!(
        held.action(&key("ctrl-enter")),
        Some(&Action::Cd(side.clone()))
    );
    assert_eq!(
        held.action(&key("ctrl-x")),
        Some(&confirm(
            &format!("Remove the worktree of side at {side}?"),
            format!("wt -C {root} remove side")
        ))
    );
    let _ = std::fs::remove_dir_all(&side);
}

#[test]
fn remote_branches_tags_and_worktrees_do_what_suits_them() {
    let (repo, side) = refs("branches-others");

    let rows = branches(&repo);

    let remote = row_of(&rows, "origin/topic");
    assert_eq!(
        remote.action(&key("ctrl-enter")),
        Some(&next("checkout", &["remote", "origin/topic"], None, None))
    );
    assert_eq!(remote.action(&key("ctrl-e")), None);
    let tag = row_of(&rows, "v1");
    assert_eq!(
        tag.action(&key("ctrl-enter")),
        Some(&next("checkout", &["tag", "v1"], None, None))
    );
    let tree = row_of(&rows, &side);
    assert_eq!(
        tree.action(&key("ctrl-enter")),
        Some(&Action::Cd(side.clone()))
    );
    let state = format!("{{\"worktree\":\"{side}\"}}");
    assert_eq!(
        tree.action(&key("ctrl-s")),
        Some(&next("contents", &[], None, Some(&state)))
    );
    let _ = std::fs::remove_dir_all(&side);
}

#[test]
fn worktrunks_status_shows_on_local_branches_when_wt_is_installed() {
    if on_path("wt", std::env::var_os("PATH").as_deref()).is_none() {
        return;
    }
    let (repo, side) = refs("branches-worktrunk");

    let rows = branches(&repo);

    assert!(
        row_of(&rows, "main").label.contains('^'),
        "{}",
        row_of(&rows, "main").label
    );
    let _ = std::fs::remove_dir_all(&side);
}

#[test]
fn pull_asks_which_way_to_pull_into_the_current_branch() {
    let (repo, side) = refs("branches-pull");
    let root = repo.0.display();

    let (header, rows) = read(&printed("pull", &[], None, &repo.places()).expect("rows"));

    assert_eq!(header, Some("Pull into main?".to_string()));
    let labels: Vec<&str> = rows.iter().map(|row| row.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            &format!("Pull  {DIM}merges as your Git settings say{RESET}")[..],
            &format!("Pull with rebase  {DIM}replays your commits on top{RESET}"),
            &format!("Cancel  {DIM}nothing changes{RESET}"),
        ]
    );
    assert_eq!(
        rows[1].enter,
        Action::Run(format!("git -C {root} pull --rebase"))
    );
    let _ = std::fs::remove_dir_all(&side);
}
