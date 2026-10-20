use herdr_picker_domain::{Action, Preview};

use crate::builtin::git::fixture::{GitRepo, labels, read};
use crate::builtin::tests::printed;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn branched(name: &str) -> GitRepo {
    let repo = GitRepo::new(name);
    repo.write("a.txt", "one\n");
    repo.commit("first");
    repo.git(&["branch", "other"]);
    repo.git(&["checkout", "-q", "other"]);
    repo.write("a.txt", "other\n");
    repo.commit("on other");
    repo.git(&["checkout", "-q", "main"]);
    repo
}

fn checkout(
    repo: &GitRepo,
    kind: &str,
    target: &str,
) -> (Option<String>, Vec<herdr_picker_domain::PickerRow>) {
    read(&printed("checkout", &[kind, target], None, &repo.places()).expect("rows"))
}

#[test]
fn a_clean_tree_asks_once_before_checking_out_and_shows_where_it_landed() {
    let repo = branched("checkout-clean");
    let root = repo.0.display();

    let (header, rows) = checkout(&repo, "local", "other");

    assert_eq!(header, Some("Check out other?".to_string()));
    assert_eq!(
        labels(&rows),
        vec![
            "Check out other".to_string(),
            format!("Cancel  {DIM}nothing changes{RESET}")
        ]
    );
    assert_eq!(
        rows[0].enter,
        Action::Run(format!(
            "git -C {root} checkout other && git -C {root} status --short --branch"
        ))
    );
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text("Check out other?".to_string()))
    );
    assert_eq!(rows[1].enter, Action::Back);
}

#[test]
fn changes_in_the_way_are_counted_named_and_offered_a_stash_first() {
    let repo = branched("checkout-dirty");
    repo.write("a.txt", "mine\n");
    let root = repo.0.display();

    let (header, rows) = checkout(&repo, "local", "other");

    assert_eq!(
        header,
        Some("Check out other? You have 1 changed file.".to_string())
    );
    assert_eq!(
        labels(&rows),
        vec![
            format!("Check out anyway  {DIM}Git will refuse: a.txt differs in other{RESET}"),
            format!("Stash, then check out  {DIM}saved as \"before checkout of other\"{RESET}"),
            format!("Cancel  {DIM}nothing changes{RESET}"),
        ]
    );
    assert_eq!(
        rows[1].enter,
        Action::Run(format!(
            "git -C {root} stash push -m 'before checkout of other' && git -C {root} checkout other && git -C {root} status --short --branch"
        ))
    );
}

#[test]
fn a_remote_branch_becomes_a_tracking_branch_and_a_commit_a_detached_head() {
    let repo = branched("checkout-kinds");
    let root = repo.0.display();
    repo.git(&["update-ref", "refs/remotes/origin/topic", "other"]);
    let first = repo.git(&["rev-parse", "main"]);

    let (_, remote) = checkout(&repo, "remote", "origin/topic");
    let (_, local_exists) = checkout(&repo, "remote", "origin/other");
    let (commit, _) = checkout(&repo, "commit", &first);

    assert_eq!(
        remote[0].enter,
        Action::Run(format!(
            "git -C {root} checkout --track origin/topic && git -C {root} status --short --branch"
        ))
    );
    assert_eq!(local_exists[0].label, "Check out other");
    assert_eq!(
        commit,
        Some(format!(
            "Check out {} \"first\" (detached HEAD)?",
            &first[..7]
        ))
    );
}
