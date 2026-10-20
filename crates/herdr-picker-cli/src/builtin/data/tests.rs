use std::fs;

use herdr_picker_domain::{Action, Key, Location, PickerCall, PickerRow, Preview, footer_hint};

use crate::builtin::Asked;
use crate::builtin::git::fixture::{labels, read};
use crate::builtin::tests::{Tree, asked, printed};

pub(super) const DIM: &str = "\x1b[2m";
pub(super) const RESET: &str = "\x1b[0m";

pub(super) const ACCOUNT: &str = r#"{
  "workspace": {
    "ignoreGlobs": ["*.log", "dist"]
  },
  "password": "hunter2",
  "payload": "eyJ1c2VyIjoiYW5uIn0="
}
"#;

// A folder holding one data file with this text.
pub(super) fn holding(name: &str, file: &str, text: &str) -> Tree {
    let tree = Tree::new(name, &[]);
    fs::write(tree.0.join(file), text).expect("a data file");
    tree
}

pub(super) fn file_rows(
    tree: &Tree,
    file: &str,
    source: &str,
    state: Option<&str>,
) -> (Option<String>, Vec<PickerRow>) {
    let path = tree.path(file);
    let asked_for = Asked {
        source: Some(source),
        state,
        query: "",
    };
    read(&asked("data-file", &[&path], &asked_for, &tree.places("")).expect("rows"))
}

pub(super) fn on(row: &PickerRow, name: &str) -> Action {
    row.action(&Key::parse(name).expect("a key"))
        .cloned()
        .unwrap_or(Action::Nothing)
}

pub(super) fn next(name: &str, file: &str, state: Option<&str>) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: vec![file.to_string()],
        state: state.map(str::to_string),
        query: None,
    })
}

#[test]
fn the_picker_opens_on_the_data_files_here_and_enter_opens_one() {
    let tree = Tree::new(
        "data-files",
        &["a.json", "logs/run.jsonl", "ci.yml", "notes.txt", "app.db"],
    );

    let (_, rows) = read(&printed("data", &[], None, &tree.places("")).expect("rows"));
    let mut shown = labels(&rows);
    shown.sort();

    assert_eq!(
        shown,
        vec![
            format!("\x1b[38;2;137;180;250mlogs/{RESET}run.jsonl  {DIM}json lines{RESET}"),
            format!("a.json  {DIM}json{RESET}"),
            format!("app.db  {DIM}sqlite{RESET}"),
            format!("ci.yml  {DIM}yaml{RESET}"),
        ]
    );
    let json = rows
        .iter()
        .find(|row| row.label.starts_with("a.json"))
        .expect("a.json");
    assert_eq!(json.enter, next("data-file", &tree.path("a.json"), None));
}

#[test]
fn a_files_values_are_rows_labeled_by_jq_path_with_base64_json_decoded_in_place() {
    let tree = holding("data-paths", "a.json", ACCOUNT);
    let file = tree.path("a.json");

    let (header, rows) = file_rows(&tree, "a.json", "paths", None);

    assert_eq!(header, None);
    assert_eq!(
        labels(&rows),
        vec![
            format!(".  {DIM}{{3 keys}}{RESET}"),
            format!(".workspace  {DIM}{{1 key}}{RESET}"),
            format!(".workspace.ignoreGlobs  {DIM}[2 items]{RESET}"),
            format!(".workspace.ignoreGlobs[0]  {DIM}\"*.log\"{RESET}"),
            format!(".workspace.ignoreGlobs[1]  {DIM}\"dist\"{RESET}"),
            format!(".password  {DIM}\"hunter2\"{RESET}"),
            format!(".payload  {DIM}{{1 key}}{RESET}  {DIM}decoded{RESET}"),
            format!(".payload.user  {DIM}\"ann\"{RESET}  {DIM}decoded{RESET}"),
        ]
    );
    let glob = &rows[3];
    assert_eq!(
        glob.enter,
        Action::Insert(".workspace.ignoreGlobs[0]".to_string())
    );
    assert_eq!(glob.preview, Some(Preview::Text("*.log".to_string())));
    assert_eq!(
        on(glob, "ctrl-enter"),
        Action::Edit(vec![Location {
            path: file.clone(),
            line: Some(3)
        }])
    );
    assert_eq!(on(glob, "alt-y"), Action::CopyStay("*.log".to_string()));
    assert_eq!(
        on(glob, "alt-;"),
        next("data-copy", &file, Some(r#"["workspace","ignoreGlobs",0]"#))
    );
    let user = &rows[7];
    assert_eq!(
        user.enter,
        Action::Insert(".payload | @base64d | fromjson | .user".to_string())
    );
    assert_eq!(
        on(user, "ctrl-enter"),
        Action::Edit(vec![Location {
            path: file,
            line: Some(6)
        }])
    );
}

#[test]
fn a_secret_copies_hidden_on_alt_y_and_visible_on_alt_u() {
    let tree = holding("data-secret", "a.json", ACCOUNT);

    let (_, rows) = file_rows(&tree, "a.json", "paths", None);

    assert_eq!(
        on(&rows[5], "alt-y"),
        Action::CopyStay("[hidden]".to_string())
    );
    assert_eq!(
        on(&rows[5], "alt-u"),
        Action::CopyStay("hunter2".to_string())
    );
    assert_eq!(on(&rows[3], "alt-u"), Action::Nothing);
    assert_eq!(
        footer_hint(&rows[5]),
        [
            "enter insert",
            "alt-; copy as JSON, YAML or another format",
            "ctrl-enter edit",
            "alt-u copy and stay, with secrets",
            "alt-y copy and stay, secrets hidden",
            "ctrl-q quickfix",
            "alt-o send",
            "F1 help"
        ]
    );
}

#[test]
fn a_long_string_previews_with_its_line_breaks_and_opens_as_plain_text() {
    let tree = holding(
        "data-text",
        "t.jsonl",
        "{\"say\":\"one\\ntwo\"}\n{\"say\":\"three\"}\n",
    );

    let (_, rows) = file_rows(&tree, "t.jsonl", "paths", None);

    assert_eq!(
        labels(&rows)[2],
        format!(".[0].say  {DIM}\"one\\ntwo\"{RESET}")
    );
    assert_eq!(rows[2].preview, Some(Preview::Text("one\ntwo".to_string())));
    let Action::Run(command) = on(&rows[2], "ctrl-enter") else {
        panic!("a command");
    };
    assert!(
        command.ends_with(&format!(
            " data value {} '[0,\"say\"]' | nvim -",
            tree.path("t.jsonl")
        )),
        "{command}"
    );
    assert_eq!(
        on(&rows[4], "ctrl-enter"),
        Action::Edit(vec![Location {
            path: tree.path("t.jsonl"),
            line: Some(2)
        }])
    );
}

#[test]
fn yaml_is_read_through_yq_into_the_same_rows() {
    let tree = holding(
        "data-yaml",
        "ci.yml",
        "jobs:\n  build:\n    runs: [linux]\n",
    );

    let (_, rows) = file_rows(&tree, "ci.yml", "paths", None);

    assert_eq!(
        labels(&rows)[3..],
        [
            format!(".jobs.build.runs  {DIM}[1 item]{RESET}"),
            format!(".jobs.build.runs[0]  {DIM}\"linux\"{RESET}"),
        ]
    );
    assert_eq!(
        on(&rows[4], "ctrl-enter"),
        Action::Edit(vec![Location {
            path: tree.path("ci.yml"),
            line: Some(3)
        }])
    );
}

#[test]
fn toon_is_read_in_process_into_the_same_rows() {
    let tree = holding(
        "data-toon",
        "users.toon",
        "users[2]{name,id}:\n  ann,1\n  bob,2\n",
    );

    let (_, rows) = file_rows(&tree, "users.toon", "paths", None);

    assert_eq!(
        labels(&rows),
        vec![
            format!(".  {DIM}{{1 key}}{RESET}"),
            format!(".users  {DIM}[2 items]{RESET}"),
            format!(".users[0]  {DIM}{{2 keys}}{RESET}"),
            format!(".users[0].name  {DIM}\"ann\"{RESET}"),
            format!(".users[0].id  {DIM}1{RESET}"),
            format!(".users[1]  {DIM}{{2 keys}}{RESET}"),
            format!(".users[1].name  {DIM}\"bob\"{RESET}"),
            format!(".users[1].id  {DIM}2{RESET}"),
        ]
    );
}

#[test]
fn a_file_that_does_not_parse_says_so_in_a_row() {
    let tree = holding("data-broken", "a.json", "{\"open\": ");

    let (_, rows) = file_rows(&tree, "a.json", "paths", None);

    assert_eq!(
        labels(&rows),
        vec![format!("{DIM}Could not read this document{RESET}")]
    );
}

#[test]
fn temporal_is_a_source_with_a_row_saying_it_is_not_built_yet() {
    let tree = Tree::new("data-temporal", &[]);

    let (_, rows) = read(&printed("data", &[], Some("temporal"), &tree.places("")).expect("rows"));

    assert_eq!(
        labels(&rows),
        vec![format!("{DIM}Temporal picker not built yet{RESET}")]
    );
}
