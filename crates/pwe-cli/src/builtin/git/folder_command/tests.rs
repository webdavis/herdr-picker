use std::ffi::OsStr;
use std::process::Command;

use super::{git_in_folder, wt_in_folder};
use crate::builtin::git::fixture::GitRepo;

fn variables_git_keeps_local_to_a_repository() -> Vec<String> {
    let listed = Command::new("git")
        .args(["rev-parse", "--local-env-vars"])
        .output()
        .expect("git runs");
    String::from_utf8_lossy(&listed.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn assert_ignores_variables_local_to_a_repository(command: &Command) {
    let removed: Vec<&OsStr> = command
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| name)
        .collect();
    let variables = variables_git_keeps_local_to_a_repository();
    assert!(variables.iter().any(|name| name == "GIT_DIR"));
    for name in &variables {
        assert!(
            removed.contains(&OsStr::new(name)),
            "{name} reaches {:?}",
            command.get_program()
        );
    }
}

#[test]
fn a_git_command_for_a_folder_ignores_an_inherited_git_dir() {
    let repo = GitRepo::new("folder-command");
    assert_ignores_variables_local_to_a_repository(&git_in_folder(&repo.0));
}

#[test]
fn a_worktrunk_command_for_a_folder_ignores_an_inherited_git_dir() {
    let repo = GitRepo::new("folder-command-worktrunk");
    let command = wt_in_folder(&repo.0);
    assert_eq!(command.get_program(), "wt");
    assert_ignores_variables_local_to_a_repository(&command);
}

#[test]
fn a_git_command_for_a_folder_acts_on_that_folder() {
    let repo = GitRepo::new("folder-command-target");
    let other = GitRepo::new("folder-command-other");
    let ran = git_in_folder(&repo.0)
        .args(["config", "pwe.marker", "yes"])
        .output()
        .expect("git runs");
    assert!(ran.status.success(), "{ran:?}");
    assert_eq!(repo.git(&["config", "pwe.marker"]), "yes");
    let untouched = git_in_folder(&other.0)
        .args(["config", "pwe.marker"])
        .output()
        .expect("git runs");
    assert!(!untouched.status.success());
}
