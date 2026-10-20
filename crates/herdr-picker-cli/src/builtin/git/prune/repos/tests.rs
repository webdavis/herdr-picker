use std::path::{Path, PathBuf};

use herdr_picker_adapters::on_path;
use herdr_picker_domain::{Action, Preview};

use super::{main_repo, worktree_base};
use crate::builtin::git::fixture::{GitRepo, read};
use crate::builtin::tests::printed;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

#[test]
fn worktrunks_worktree_folder_is_the_fixed_start_of_its_path_template() {
    let home = Some(Path::new("/home/me"));

    assert_eq!(
        worktree_base("~/w/.worktrees/{{ repo }}/{{ branch | sanitize }}", home),
        Some(PathBuf::from("/home/me/w/.worktrees"))
    );
    assert_eq!(
        worktree_base("/srv/trees/{{ repo }}", home),
        Some(PathBuf::from("/srv/trees"))
    );
    assert_eq!(worktree_base("../{{ repo }}.{{ branch }}", home), None);
    assert_eq!(worktree_base("~/{{ repo }}", None), None);
}

#[test]
fn a_worktree_leads_back_to_the_repository_it_belongs_to() {
    let repo = GitRepo::new("prune-main");
    repo.write("a.txt", "one\n");
    repo.commit("first");
    let side = format!("{}-side", repo.0.display());
    let _ = std::fs::remove_dir_all(&side);
    repo.git(&["worktree", "add", "-q", "-b", "side", &side]);

    assert_eq!(main_repo(Path::new(&side)), Some(repo.0.clone()));
    assert_eq!(main_repo(&repo.0), Some(repo.0.clone()));
    let _ = std::fs::remove_dir_all(&side);
}

#[test]
fn the_prune_step_shows_what_worktrunk_would_remove_and_asks_first() {
    if on_path("wt", std::env::var_os("PATH").as_deref()).is_none() {
        return;
    }
    let repo = GitRepo::new("prune-step");
    repo.write("a.txt", "one\n");
    repo.commit("first");
    let root = repo.0.display().to_string();

    let (header, rows) = read(&printed("prune", &[&root], None, &repo.places()).expect("rows"));

    assert_eq!(header, Some(format!("Prune {root}?")));
    assert_eq!(
        rows[0].label,
        format!("Prune  {DIM}removes what the list shows{RESET}")
    );
    assert_eq!(
        rows[0].enter,
        Action::Run(format!("wt -C {root} step prune"))
    );
    let Some(Preview::Text(preview)) = &rows[0].preview else {
        panic!("a text preview");
    };
    assert!(preview.starts_with(&format!("{root}\n")), "{preview}");
    assert_eq!(rows[1].enter, Action::Back);
}
