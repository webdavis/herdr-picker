use std::fs;

use herdr_picker_adapters::Capture;
use herdr_picker_domain::{Action, Key, Location, PickerRow, Preview};

use crate::builtin::git::fixture::{labels, read};
use crate::builtin::project::capture::saved_dir;
use crate::builtin::recipes::tests::project;
use crate::builtin::tests::Tree;
use crate::builtin::tests::printed;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const YELLOW: &str = "\x1b[38;2;249;226;175m";

const CARGO_CHECK: &str = concat!(
    r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable: `x`","rendered":"warning: unused variable: `x`\n --> src/main.rs:2:9\n","spans":[{"file_name":"src/main.rs","line_start":2,"column_start":9,"is_primary":true}]}}"#,
    "\n",
    r#"{"reason":"build-finished","success":false}"#,
    "\n\x1b[1msrc/lib.rs:7:3: error: broken\x1b[0m\n"
);

fn checked(name: &str) -> Tree {
    let tree = project(name);
    fs::write(tree.0.join("Cargo.toml"), "[package]\n").expect("a manifest");
    let places = tree.places("home");
    let saved = Capture {
        category: "diagnostics".to_string(),
        cwd: tree.0.to_string_lossy().into_owned(),
        framework: "rust".to_string(),
        argv: ["cargo", "check"].map(str::to_string).to_vec(),
        exit_code: 101,
        stdout: CARGO_CHECK.to_string(),
        stderr: "plain noise\n".to_string(),
    };
    let file = saved_dir(&places).join("diagnostics-rust.json");
    fs::create_dir_all(file.parent().expect("a folder")).expect("the saved folder");
    fs::write(file, serde_json::to_string(&saved).expect("json")).expect("a saved check");
    tree
}

fn errors(tree: &Tree) -> Vec<PickerRow> {
    read(&printed("tests", &[], Some("errors"), &tree.places("home")).expect("rows")).1
}

#[test]
fn a_saved_check_lists_its_messages_then_the_checks_that_can_run_again() {
    let tree = checked("errors-list");

    assert_eq!(
        labels(&errors(&tree)),
        vec![
            format!("{DIM}rust: exit 101{RESET}"),
            format!("{YELLOW}warning{RESET}: unused variable: `x`"),
            "src/lib.rs:7:3: error: broken".to_string(),
            "plain noise".to_string(),
            format!("Run Cargo check and save diagnostics  {DIM}save diagnostics{RESET}"),
            format!("build  {DIM}save diagnostics{RESET}"),
            format!("sub::lint  {DIM}save diagnostics{RESET}"),
        ]
    );
}

#[test]
fn a_message_at_a_place_opens_there_and_one_without_is_copied() {
    let tree = checked("errors-places");
    let rows = errors(&tree);

    let at = |path: &str, line| {
        Action::Edit(vec![Location {
            path: tree.path(path),
            line: Some(line),
        }])
    };
    assert_eq!(rows[1].enter, at("src/main.rs", 2));
    assert_eq!(
        rows[1].preview,
        Some(Preview::Text(
            "warning: unused variable: `x`\n --> src/main.rs:2:9\n".to_string()
        ))
    );
    assert_eq!(rows[2].enter, at("src/lib.rs", 7));
    assert_eq!(rows[3].enter, Action::Copy("plain noise".to_string()));
}

#[test]
fn a_check_is_prepared_or_run_through_herdr_picker_capture_into_the_saved_folder() {
    let tree = checked("errors-capture");
    let places = tree.places("home");
    let rows = errors(&tree);

    let output = saved_dir(&places).join("diagnostics-rust.json");
    let Action::Insert(command) = &rows[4].enter else {
        panic!("a prepared command: {:?}", rows[4].enter);
    };
    assert!(
        command.ends_with(&format!(
            " capture {} {} diagnostics rust -- cargo check --manifest-path {} --message-format=json",
            output.display(),
            tree.0.display(),
            tree.path("Cargo.toml")
        )),
        "{command}"
    );
    assert_eq!(
        rows[4].action(&Key::parse("ctrl-enter").expect("a key")),
        Some(&Action::Run(command.clone()))
    );
}

#[test]
fn with_nothing_saved_and_nothing_to_check_one_row_says_so() {
    let tree = Tree::new("errors-none", &[]);

    assert_eq!(
        labels(&errors(&tree)),
        vec![format!("{DIM}No saved results{RESET}")]
    );
}
