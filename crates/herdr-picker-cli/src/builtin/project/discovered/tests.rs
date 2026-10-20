use std::fs;

use herdr_picker_adapters::Capture;
use herdr_picker_domain::{Action, PickerRow, Preview};

use crate::builtin::git::fixture::{labels, read};
use crate::builtin::project::capture::saved_dir;
use crate::builtin::tests::{Tree, printed};

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn save(tree: &Tree, framework: &str, exit_code: i32, stdout: &str) {
    let saved = Capture {
        category: "tests".to_string(),
        cwd: tree.0.to_string_lossy().into_owned(),
        framework: framework.to_string(),
        argv: vec!["discover".to_string()],
        exit_code,
        stdout: stdout.to_string(),
        stderr: "noise\n".to_string(),
    };
    let file = saved_dir(&tree.places("home")).join(format!("tests-{framework}.json"));
    fs::create_dir_all(file.parent().expect("a folder")).expect("the saved folder");
    fs::write(file, serde_json::to_string(&saved).expect("json")).expect("a saved listing");
}

fn tests(tree: &Tree) -> Vec<PickerRow> {
    read(&printed("tests", &[], Some("tests"), &tree.places("home")).expect("rows")).1
}

fn entered(rows: &[PickerRow]) -> Vec<String> {
    rows.iter()
        .filter_map(|row| match &row.enter {
            Action::Insert(command) => Some(command.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn each_saved_listing_s_tests_run_one_at_a_time() {
    let tree = Tree::new("discovered-list", &["Cargo.toml"]);
    save(
        &tree,
        "go",
        0,
        "TestAdds\nExampleSum\nok  \texample.com/m\t0.1s\n",
    );
    save(
        &tree,
        "pytest",
        0,
        "tests/test_math.py::test_adds[1-2]\n\n1 test collected in 0.01s\n",
    );
    save(
        &tree,
        "rust",
        0,
        "math::adds: test\nmath::adds: test\nmath::bench: benchmark\n",
    );
    save(
        &tree,
        "swift",
        0,
        "Building for debugging...\nMathTests.MathTests/testAdds\nMathTests.Suite/adds(_:)\n",
    );
    let at = |inside: &str| tree.path(inside);
    let folder = tree.0.display();

    let rows = tests(&tree);

    assert_eq!(
        labels(&rows[..7]),
        vec![
            "TestAdds",
            "ExampleSum",
            "tests/test_math.py::test_adds[1-2]",
            "math::adds",
            "MathTests.MathTests/testAdds",
            "MathTests.Suite/adds(_:)",
            "Discover Rust tests  \x1b[2msave tests\x1b[0m",
        ]
    );
    assert_eq!(
        entered(&rows[..6]),
        vec![
            format!("go -C {} test ./... -run '^TestAdds$'", folder),
            format!("go -C {} test ./... -run '^ExampleSum$'", folder),
            format!(
                "python3 -m pytest '{}'",
                at("tests/test_math.py::test_adds[1-2]")
            ),
            format!(
                "cargo test --manifest-path {} math::adds -- --exact",
                at("Cargo.toml")
            ),
            format!(
                "swift test --package-path {} --filter '^MathTests\\.MathTests/testAdds$'",
                folder
            ),
            format!(
                "swift test --package-path {} --filter '^MathTests\\.Suite/adds\\(_:\\)$'",
                folder
            ),
        ]
    );
}

#[test]
fn a_failed_listing_says_so_with_its_output() {
    let tree = Tree::new("discovered-failed", &[]);
    save(&tree, "rust", 101, "error: could not compile\n");

    let rows = tests(&tree);

    assert_eq!(
        labels(&rows[..1]),
        vec![format!("{DIM}Discovery failed: rust, exit 101{RESET}")]
    );
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "error: could not compile\n\nnoise\n".to_string()
        ))
    );
}

#[test]
fn each_kind_of_project_can_save_a_listing_of_its_tests() {
    let tree = Tree::new(
        "discovered-capture",
        &["Package.swift", "Cargo.toml", "pyproject.toml", "go.mod"],
    );
    let saved = saved_dir(&tree.places("home"));
    let capture = |framework: &str| {
        format!(
            " capture {} {} tests {framework} -- ",
            saved.join(format!("tests-{framework}.json")).display(),
            tree.0.display()
        )
    };

    let rows = tests(&tree);

    assert_eq!(
        labels(&rows),
        vec![
            format!("Discover Swift tests (XCTest and Swift Testing)  {DIM}save tests{RESET}"),
            "Run all Swift tests".to_string(),
            format!("Discover Rust tests  {DIM}save tests{RESET}"),
            "Run all Rust tests".to_string(),
            format!("Discover pytest tests  {DIM}save tests{RESET}"),
            format!("Discover Go tests  {DIM}save tests{RESET}"),
        ]
    );
    let commands = entered(&rows);
    let expected = [
        (
            0,
            "swift",
            format!("swift test --package-path {} list", tree.0.display()),
        ),
        (
            2,
            "rust",
            format!(
                "cargo test --manifest-path {} -- --list --format terse",
                tree.path("Cargo.toml")
            ),
        ),
        (
            4,
            "pytest",
            "python3 -m pytest --collect-only -q".to_string(),
        ),
        (5, "go", "go test ./... -list .".to_string()),
    ];
    for (index, framework, listing) in expected {
        let wanted = format!("{}{listing}", capture(framework));
        assert!(commands[index].ends_with(&wanted), "{}", commands[index]);
    }
}
