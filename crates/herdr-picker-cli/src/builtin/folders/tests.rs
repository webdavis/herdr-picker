use std::fs;
use std::process::Command;

use crate::builtin::tests::{Tree, printed};

fn folders(source: &str, places: &crate::builtin::Places) -> Vec<String> {
    printed("folders", &[], Some(source), places)
        .expect("rows")
        .lines()
        .map(|line| {
            let row: serde_json::Value = serde_json::from_str(line).expect("a row");
            row["enter"]["cd"].as_str().expect("a cd").to_string()
        })
        .collect()
}

#[test]
fn subfolders_are_every_folder_below_this_one_and_enter_changes_into_it() {
    let tree = Tree::new(
        "folders-sub",
        &["src/deep/x.rs", "docs/a.md", ".git/config"],
    );

    assert_eq!(
        folders("subfolders", &tree.places("")),
        [tree.path("docs"), tree.path("src"), tree.path("src/deep")]
    );
    let rows = printed("folders", &[], None, &tree.places("")).expect("rows");
    assert_eq!(
        rows.lines().next(),
        Some(
            format!(
                r#"{{"label":"\u001b[38;2;137;180;250mdocs\u001b[0m","preview":{{"path":"{docs}"}},"enter":{{"cd":"{docs}"}},"keys":{{"alt-y":{{"copy_stay":"docs"}},"ctrl-o":{{"open":"{docs}"}}}}}}"#,
                docs = tree.path("docs")
            )
            .as_str()
        )
    );
}

#[test]
fn parents_climb_from_this_folder_to_the_root() {
    let tree = Tree::new("folders-parents", &["a/b/c.txt"]);
    let mut places = tree.places("");
    places.cwd = tree.0.join("a/b");

    let parents = folders("parents", &places);

    assert_eq!(
        parents[..2],
        [tree.path("a"), tree.0.to_string_lossy().into_owned()]
    );
    assert_eq!(parents.last().map(String::as_str), Some("/"));
}

#[test]
fn git_lists_every_folder_holding_a_tracked_file_and_the_repository_root() {
    let tree = Tree::new(
        "folders-git",
        &["src/deep/x.rs", "README.md", "untracked/y.txt"],
    );
    let git = |arguments: &[&str]| {
        let ran = Command::new("git")
            .arg("-C")
            .arg(&tree.0)
            .args(arguments)
            .output()
            .expect("git runs");
        assert!(ran.status.success(), "{ran:?}");
    };
    git(&["init", "--quiet"]);
    git(&["add", "src", "README.md"]);
    let mut places = tree.places("");
    places.cwd = tree.0.join("src");

    assert_eq!(
        folders("git", &places),
        [
            tree.0.to_string_lossy().into_owned(),
            tree.path("src"),
            tree.path("src/deep")
        ]
    );
}

#[test]
fn outside_a_repository_git_lists_nothing() {
    let tree = Tree::new("folders-no-git", &["a/x.txt"]);

    assert_eq!(folders("git", &tree.places("")), Vec::<String>::new());
}

#[test]
fn bookmarks_are_the_saved_folders_that_still_exist() {
    let tree = Tree::new("folders-bookmarks", &["kept/x.txt"]);
    let places = tree.places("");
    fs::create_dir_all(places.bookmarks.parent().expect("a parent")).expect("a folder");
    fs::write(
        &places.bookmarks,
        format!("{}\n/nonexistent/gone\n", tree.path("kept")),
    )
    .expect("bookmarks");

    assert_eq!(folders("bookmarks", &places), [tree.path("kept")]);
}
