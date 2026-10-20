use std::fs::File;
use std::time::{Duration, SystemTime};

use herdr_picker_adapters::on_path;

use crate::builtin::tests::{Tree, printed};

fn files(source: &str, tree: &Tree, home: &str) -> String {
    printed("files", &[], Some(source), &tree.places(home)).expect("rows")
}

// Home is beside the current folder, as it is when the picker runs in a project.
fn files_from_work(source: &str, tree: &Tree) -> String {
    let mut places = tree.places("me");
    places.cwd = tree.0.join("work");
    printed("files", &[], Some(source), &places).expect("rows")
}

fn inserted(rows: &str) -> Vec<String> {
    rows.lines()
        .map(|line| {
            let row: serde_json::Value = serde_json::from_str(line).expect("a row");
            row["enter"]["insert"]
                .as_str()
                .expect("an insert")
                .to_string()
        })
        .collect()
}

#[test]
fn this_folder_lists_its_files_by_relative_path_with_the_files_picker_keys() {
    let tree = Tree::new("files-here", &["src/main.rs", ".env"]);
    let main = tree.path("src/main.rs");

    assert_eq!(
        files("this folder", &tree, ""),
        format!(
            concat!(
                r#"{{"label":"{blue}src/{reset}main.rs","preview":{{"path":"{main}"}},"enter":{{"insert":"src/main.rs"}},"#,
                r#""keys":{{"alt-.":{{"source":"hidden"}},"alt-c":{{"cd":"{src}"}},"alt-enter":{{"edit":[{{"path":"{main}"}}]}},"#,
                r#""alt-y":{{"copy_stay":"src/main.rs"}},"ctrl-o":{{"open":"{main}"}}}}}}"#,
                "\n"
            ),
            blue = "\\u001b[38;2;137;180;250m",
            reset = "\\u001b[0m",
            main = main,
            src = tree.path("src"),
        )
    );
}

#[test]
fn hidden_adds_dotfiles_and_alt_dot_goes_back_to_this_folder() {
    let tree = Tree::new("files-hidden", &["a b.txt", ".env"]);

    let rows = files("hidden", &tree, "");

    assert_eq!(inserted(&rows), [".env", "'a b.txt'"]);
    assert!(
        rows.contains(r#""alt-.":{"source":"this folder"}"#),
        "{rows}"
    );
}

#[test]
fn home_lists_two_levels_of_home_by_full_path() {
    let tree = Tree::new(
        "files-home",
        &["me/notes.txt", "me/docs/plan.md", "me/docs/old/deep.md"],
    );

    let rows = files_from_work("home", &tree);

    assert_eq!(
        inserted(&rows),
        [tree.path("me/docs/plan.md"), tree.path("me/notes.txt")]
    );
    assert!(!rows.contains("alt-."), "{rows}");
}

#[test]
fn recent_lists_documents_and_downloads_newest_first_named_then_dimmed_folder() {
    let tree = Tree::new(
        "files-recent",
        &[
            "me/Documents/old.pdf",
            "me/Downloads/new.zip",
            "me/Desktop/skip.txt",
        ],
    );
    let aged = |inside: &str, seconds_ago: u64| {
        let file = File::options()
            .write(true)
            .open(tree.0.join(inside))
            .expect("a file");
        file.set_modified(SystemTime::now() - Duration::from_secs(seconds_ago))
            .expect("a modified time");
    };
    aged("me/Documents/old.pdf", 3600);
    aged("me/Downloads/new.zip", 60);

    let rows = files_from_work("recent", &tree);

    assert_eq!(
        inserted(&rows),
        [
            tree.path("me/Downloads/new.zip"),
            tree.path("me/Documents/old.pdf")
        ]
    );
    let first: serde_json::Value =
        serde_json::from_str(rows.lines().next().expect("a row")).expect("a row");
    assert_eq!(first["label"], "new.zip  \x1b[2m~/Downloads\x1b[0m");
}

#[test]
fn an_unknown_source_is_refused() {
    let tree = Tree::new("files-unknown", &[]);

    let refusal = printed("files", &[], Some("sideways"), &tree.places(""));

    assert_eq!(
        refusal.map_err(|failure| format!("{failure:?}")),
        Err("Failed(\"the files picker has no source named sideways\")".to_string())
    );
}

#[test]
fn with_fd_installed_the_same_files_are_listed() {
    let tree = Tree::new("files-fd", &["a.txt", "src/main.rs", "src/lib.rs"]);
    let Some(fd) = on_path("fd", std::env::var_os("PATH").as_deref()) else {
        eprintln!("fd is not installed; only the walk without it is checked");
        return;
    };
    let mut places = tree.places("");
    places.fd = Some(fd);
    let sorted = |rows: String| {
        let mut lines: Vec<String> = rows.lines().map(str::to_string).collect();
        lines.sort();
        lines
    };

    assert_eq!(
        sorted(printed("files", &[], None, &places).expect("rows")),
        sorted(files("this folder", &tree, ""))
    );
}
