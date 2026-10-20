use std::fs;

use herdr_picker_domain::{Action, PickerRow, Preview};

use crate::builtin::git::fixture::{labels, read};
use crate::builtin::tests::{Tree, printed};

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const PACKAGE: &str = r#"{"scripts": {"test": "jest", "build": "tsc", "Test:e2e": "playwright test", "test:odd": 3}}"#;

fn tests(tree: &Tree) -> Vec<PickerRow> {
    read(&printed("tests", &[], Some("tests"), &tree.places("home")).expect("rows")).1
}

#[test]
fn the_package_s_test_scripts_run_with_npm_in_its_folder() {
    let tree = Tree::new("packages-npm", &[]);
    fs::write(tree.0.join("package.json"), PACKAGE).expect("a package");

    let rows = tests(&tree);

    assert_eq!(labels(&rows), vec!["npm: test", "npm: Test:e2e"]);
    let command = format!("npm --prefix {} run Test:e2e", tree.0.display());
    assert_eq!(rows[1].enter, Action::Insert(command.clone()));
    assert_eq!(
        rows[1].preview,
        Some(Preview::Text(format!(
            "{command}\n\nplaywright test\nEnter prepares this command."
        )))
    );
}

#[test]
fn a_package_that_cannot_be_read_says_so() {
    let tree = Tree::new("packages-broken", &[]);
    fs::write(tree.0.join("package.json"), "{ not json").expect("a package");

    let rows = tests(&tree);

    assert_eq!(
        labels(&rows),
        vec![format!("{DIM}Invalid package.json{RESET}")]
    );
}

#[test]
fn each_project_below_moves_there_with_the_files_that_mark_it() {
    let tree = Tree::new(
        "packages-nested",
        &[
            "crates/a/Cargo.toml",
            "crates/a/inner/go.mod",
            "web/package.json",
            "web/go.mod",
            "notes/readme.md",
        ],
    );
    fs::write(tree.0.join("web/package.json"), "{}").expect("a package");

    let rows = tests(&tree);

    assert_eq!(
        labels(&rows),
        vec![
            format!("Project: crates/a  {DIM}Cargo.toml{RESET}"),
            format!("Project: web  {DIM}go.mod, package.json{RESET}"),
        ]
    );
    assert_eq!(rows[0].enter, Action::Cd(tree.path("crates/a")));
}
