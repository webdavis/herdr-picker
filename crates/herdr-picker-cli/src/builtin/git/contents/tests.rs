use herdr_picker_domain::{Action, Key, Location, PickerRow};

use crate::builtin::Asked;
use crate::builtin::git::fixture::{GitRepo, read};
use crate::builtin::tests::{Tree, asked};

const BLUE: &str = "\x1b[38;2;137;180;250m";
const YELLOW: &str = "\x1b[38;2;249;226;175m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

fn project(name: &str) -> GitRepo {
    let repo = GitRepo::new(name);
    repo.write("src/a.rs", "fn alpha() {}\nlet beta = 1;\n");
    repo.write("README.md", "alpha docs\n");
    repo.write("tests/t.rs", "alpha test\n");
    repo.write(".hidden", "alpha hidden\n");
    repo.commit("first");
    repo.write("new.txt", "alpha new\n");
    repo
}

fn search(repo: &GitRepo, query: &str, state: Option<&str>) -> Vec<PickerRow> {
    let asking = Asked {
        state,
        query,
        ..Asked::default()
    };
    read(&asked("contents", &[], &asking, &repo.places()).expect("rows")).1
}

fn plain_labels(rows: &[PickerRow]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            row.label
                .replace(BLUE, "")
                .replace(YELLOW, "")
                .replace(RESET, "")
                .replace(DIM, "")
        })
        .collect()
}

#[test]
fn the_tracked_lines_matching_the_query_are_listed_in_file_order_under_their_area() {
    let repo = project("contents-tracked");

    let rows = search(&repo, "alpha", None);

    assert_eq!(
        plain_labels(&rows),
        vec![
            ".hidden:1: alpha hidden",
            "README.md:1: alpha docs",
            "src/a.rs:1: fn alpha() {}",
            "tests/t.rs:1: alpha test"
        ]
    );
    assert_eq!(
        rows[2].label,
        format!("{BLUE}src/a.rs{RESET}:{YELLOW}1{RESET}: fn alpha() {{}}")
    );
    let groups: Vec<Option<&str>> = rows.iter().map(|row| row.group.as_deref()).collect();
    assert_eq!(
        groups,
        vec![Some("config"), Some("docs"), Some("source"), Some("tests")]
    );
}

#[test]
fn a_line_opens_at_its_number_and_its_keys_flip_the_search_or_trace_the_line() {
    let repo = project("contents-keys");

    let rows = search(&repo, "alpha", None);
    let row = &rows[2];

    assert_eq!(
        row.enter,
        Action::Edit(vec![Location {
            path: repo.path("src/a.rs"),
            line: Some(1)
        }])
    );
    let state = |text: &str| Some(Action::State(text.to_string()));
    assert_eq!(
        row.action(&key("alt-x")).cloned(),
        state("{\"regex\":true}")
    );
    assert_eq!(
        row.action(&key("alt-u")).cloned(),
        state("{\"untracked\":true}")
    );
    assert_eq!(
        row.action(&key("alt-.")).cloned(),
        state("{\"hidden\":false}")
    );
    assert_eq!(row.action(&key("alt-s")).cloned(), state("{\"sort\":true}"));
    assert_eq!(
        row.action(&key("alt-g")),
        Some(&Action::Run(format!(
            "git -C {} log -L 1,1:src/a.rs",
            repo.0.display()
        )))
    );
}

#[test]
fn untracked_regex_and_hidden_follow_the_state() {
    let repo = project("contents-state");

    assert_eq!(
        plain_labels(&search(&repo, "alpha", Some("{\"untracked\":true}"))),
        vec!["new.txt:1: alpha new"]
    );
    assert_eq!(
        plain_labels(&search(&repo, "al.ha", None)),
        vec!["No line in the tracked files matches al.ha"]
    );
    assert_eq!(search(&repo, "al.ha", Some("{\"regex\":true}")).len(), 4);
    assert_eq!(search(&repo, "alpha", Some("{\"hidden\":false}")).len(), 3);
    assert_eq!(
        plain_labels(&search(&repo, "", Some("{\"untracked\":true}"))),
        vec!["Type to search the untracked files"]
    );
    let row = &search(&repo, "alpha", Some("{\"regex\":true,\"untracked\":true}"))[0];
    assert_eq!(
        row.action(&key("alt-x")).cloned(),
        Some(Action::State("{\"untracked\":true}".to_string()))
    );
}

#[test]
fn a_changed_file_has_no_committed_line_history_to_trace() {
    let repo = project("contents-changed");
    repo.write("README.md", "alpha docs changed\n");

    let rows = search(&repo, "alpha", None);

    assert_eq!(rows[1].action(&key("alt-g")), None);
}

#[test]
fn outside_git_every_file_here_is_searched() {
    let tree = Tree::new("contents-plain", &["notes.txt"]);
    std::fs::write(tree.0.join("notes.txt"), "an alpha line\n").expect("a file");
    let asking = Asked {
        query: "alpha",
        ..Asked::default()
    };

    let (_, rows) = read(&asked("contents", &[], &asking, &tree.places("")).expect("rows"));

    assert_eq!(plain_labels(&rows), vec!["notes.txt:1: an alpha line"]);
    assert_eq!(rows[0].action(&key("alt-u")), None);
}

#[test]
fn with_bat_a_hit_previews_its_file_from_a_little_above_the_line_with_the_line_marked() {
    let repo = project("contents-bat");
    let mut places = repo.places();
    places.bat = Some(std::path::PathBuf::from("/opt/bin/bat"));
    let asking = Asked {
        query: "beta",
        ..Asked::default()
    };

    let (_, rows) = read(&asked("contents", &[], &asking, &places).expect("rows"));

    assert_eq!(
        rows[0].preview,
        Some(herdr_picker_domain::Preview::Command(vec![
            "/opt/bin/bat".to_string(),
            "--color=always".to_string(),
            "--style=numbers".to_string(),
            "--highlight-line=2".to_string(),
            "--line-range=1:".to_string(),
            repo.path("src/a.rs"),
        ]))
    );
}

#[test]
fn with_no_line_to_show_the_switches_still_work_from_the_one_row_saying_so() {
    let repo = project("contents-nothing");

    let rows = search(&repo, "zzz", Some("{\"untracked\":true}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].enter, Action::Nothing);
    assert_eq!(
        rows[0].action(&key("alt-u")),
        Some(&Action::State("{}".to_string()))
    );
}
