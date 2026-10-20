use std::fs;

use herdr_picker_domain::{Action, Key, Location, PickerRow};

use crate::builtin::git::fixture::{labels, read};
use crate::builtin::recipes::tests::project;
use crate::builtin::tests::{Tree, printed};

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const SMOKE: &str = "#!/usr/bin/env bash\ntest_adds() {\n  true\n}\nfunction test_subtracts () {\n  true\n}\nhelper() {\n  true\n}\n";

fn tested(name: &str) -> Tree {
    let tree = project(name);
    for (file, text) in [
        ("Cargo.toml", "[package]\n"),
        ("tests/unit/run.sh", "echo run\n"),
        ("tests/unit/smoke.test.sh", SMOKE),
        ("tools/build.sh", "echo build\n"),
    ] {
        let path = tree.0.join(file);
        fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
        fs::write(path, text).expect("a file");
    }
    tree
}

fn tests(tree: &Tree) -> Vec<PickerRow> {
    read(&printed("tests", &[], Some("tests"), &tree.places("home")).expect("rows")).1
}

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

#[test]
fn the_project_s_suites_bash_tests_and_test_recipes_are_listed() {
    let tree = tested("tests-list");

    assert_eq!(
        labels(&tests(&tree)),
        vec![
            "Run all Rust tests".to_string(),
            "tests/unit/run.sh".to_string(),
            "tests/unit/smoke.test.sh: test_adds".to_string(),
            "tests/unit/smoke.test.sh: test_subtracts".to_string(),
            "test-unit".to_string(),
        ]
    );
}

#[test]
fn a_bash_test_is_prepared_or_run_with_bashunit_at_its_line_and_can_be_edited_there() {
    let tree = tested("tests-bash");
    let file = tree.path("tests/unit/smoke.test.sh");
    let rows = tests(&tree);

    let subtracts = &rows[3];
    let command = format!("bashunit {file}:5");
    assert_eq!(subtracts.enter, Action::Insert(command.clone()));
    assert_eq!(
        subtracts.action(&key("ctrl-enter")),
        Some(&Action::Run(command))
    );
    assert_eq!(
        subtracts.action(&key("alt-e")),
        Some(&Action::Edit(vec![Location {
            path: file,
            line: Some(5)
        }]))
    );
    assert_eq!(
        rows[0].enter,
        Action::Insert(format!(
            "cargo test --manifest-path {}",
            tree.path("Cargo.toml")
        ))
    );
}

#[test]
fn a_project_without_tests_says_which_kinds_are_found() {
    let tree = Tree::new("tests-none", &["notes.txt"]);

    let rows = tests(&tree);

    assert_eq!(
        labels(&rows),
        vec![format!("{DIM}No supported tests found{RESET}")]
    );
    assert_eq!(rows[0].enter, Action::Nothing);
}
