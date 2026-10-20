use std::env;
use std::fs;
use std::path::PathBuf;

use super::*;

const ROWS: &str = r#"{"label":"src/a.rs:7: oops","enter":{"edit":[{"path":"/p/src/a.rs","line":7}]}}
{"label":"just a note","enter":{"copy":"note"}}
{"label":"b.txt","enter":{"insert":"b.txt"},"keys":{"ctrl-enter":{"edit":[{"path":"/p/b.txt"}]}}}
"#;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            env::temp_dir().join(format!("hwj-cli-quickfix-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        fs::write(directory.join("rows.jsonl"), ROWS).expect("a rows cache");
        Self(directory)
    }

    fn save(&self, fields: &[&str]) -> String {
        let quickfix = Quickfix {
            file: self.0.join("quickfix.json"),
            nvim: None,
        };
        save_marked(
            &self.0.join("rows.jsonl"),
            fields,
            &quickfix,
            Theme::CatppuccinMocha,
        )
        .expect("fzf actions")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_marked_rows_with_a_file_are_saved_and_the_picker_leaves_to_open_neovim_on_them() {
    let sandbox = Sandbox::new("save");

    assert_eq!(sandbox.save(&["0 1 2"]), "print(quickfix)+accept");
    assert_eq!(
        fs::read_to_string(sandbox.0.join("quickfix.json")).expect("the list"),
        r#"[{"filename":"/p/src/a.rs","lnum":7,"text":"src/a.rs:7: oops"},{"filename":"/p/b.txt","lnum":1,"text":"b.txt"}]"#
    );
}

#[test]
fn rows_with_no_file_save_nothing_and_say_so() {
    let sandbox = Sandbox::new("none");

    assert_eq!(
        sandbox.save(&["1"]),
        "change-footer:\x1b[38;2;243;139;168mNo file locations in the selection.\x1b[0m"
    );
    assert!(!sandbox.0.join("quickfix.json").exists());
}
