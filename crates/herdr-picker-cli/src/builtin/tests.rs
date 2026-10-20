use std::fs;
use std::path::{Path, PathBuf};

use super::*;

// A folder of files under the temp directory, removed when the test ends.
pub(crate) struct Tree(pub(crate) PathBuf);

impl Tree {
    pub(crate) fn new(name: &str, files: &[&str]) -> Self {
        // The real path: macOS reaches its temp directory through a symlink.
        let temp = std::env::temp_dir()
            .canonicalize()
            .expect("the temp directory");
        let root = temp.join(format!("hwj-builtin-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("a tree");
        for file in files {
            let path = root.join(file);
            fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
            fs::write(&path, "x").expect("a file");
        }
        Self(root)
    }

    pub(crate) fn path(&self, inside: &str) -> String {
        self.0.join(inside).to_string_lossy().into_owned()
    }

    // The tree is the current folder, home is its home folder, and fd is not installed.
    pub(crate) fn places(&self, home: &str) -> Places {
        Places {
            cwd: self.0.clone(),
            home: Some(self.0.join(home)),
            fd: None,
            bookmarks: self.0.join("state/bookmarks"),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn printed(
    name: &str,
    arguments: &[&str],
    source: Option<&str>,
    places: &Places,
) -> Result<String, CommandError> {
    let mut out = Vec::new();
    print_rows(name, arguments, source, places, &mut out)?;
    Ok(String::from_utf8(out).expect("rows are text"))
}

fn unused_places() -> Places {
    Places {
        cwd: PathBuf::from("/unused"),
        home: None,
        fd: None,
        bookmarks: PathBuf::from("/unused/bookmarks"),
    }
}

#[test]
fn echo_args_prints_one_row_per_argument_that_inserts_it_as_a_shell_word() {
    assert_eq!(
        printed("echo-args", &["alpha", "my file"], None, &unused_places()).expect("rows"),
        concat!(
            r#"{"label":"alpha","enter":{"insert":"alpha"}}"#,
            "\n",
            r#"{"label":"my file","enter":{"insert":"'my file'"}}"#,
            "\n"
        )
    );
}

#[test]
fn confirm_prints_yes_running_the_action_and_no_going_back() {
    assert_eq!(
        printed(
            "confirm",
            &["Delete x?", r#"{"run":"rm x"}"#],
            None,
            &unused_places()
        )
        .expect("rows"),
        concat!(
            r#"{"label":"Yes","preview":"Delete x?","enter":{"run":"rm x"}}"#,
            "\n",
            r#"{"label":"No","preview":"Delete x?","enter":"back"}"#,
            "\n"
        )
    );
}

#[test]
fn an_unknown_builtin_or_a_bad_confirm_is_refused() {
    for (name, arguments) in [
        ("nope", &[][..]),
        ("confirm", &["Sure?", "\"sideways\""][..]),
    ] {
        let refusal = printed(name, arguments, None, &unused_places());
        assert!(
            matches!(refusal, Err(CommandError::Failed(_))),
            "{refusal:?}"
        );
    }
}

#[test]
fn bookmarking_a_folder_saves_it_once_and_says_so_in_home_terms() {
    let tree = Tree::new("bookmark", &["repo/a.txt"]);
    let places = tree.places("");
    let repo = tree.0.join("repo");

    assert_eq!(
        bookmark(&repo, &places).expect("a bookmark"),
        "Bookmarked ~/repo"
    );
    assert_eq!(
        bookmark(&repo, &places).expect("a bookmark"),
        "Already bookmarked ~/repo"
    );
    assert_eq!(
        fs::read_to_string(&places.bookmarks).expect("the bookmarks file"),
        format!("{}\n", repo.display())
    );
}

#[test]
fn from_herdr_the_focused_panes_folder_is_bookmarked_else_the_shells() {
    assert_eq!(
        bookmark_folder(Some("/work/agent".to_string()), Path::new("/plugin/root")),
        PathBuf::from("/work/agent")
    );
    assert_eq!(
        bookmark_folder(Some(String::new()), Path::new("/work/shell")),
        PathBuf::from("/work/shell")
    );
    assert_eq!(
        bookmark_folder(None, Path::new("/work/shell")),
        PathBuf::from("/work/shell")
    );
}
