use std::fs;
use std::path::PathBuf;

use super::*;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-cli-picker-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, content).expect("a sandbox file");
        path
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const CACHED_ROWS: &str = r#"{"label":"alpha","enter":{"insert":"a"}}
{"label":"beta","preview":"the second\nrow","enter":{"cd":"/opt/beta"}}
"#;

#[test]
fn preview_prints_the_preview_of_the_row_at_the_index_the_finder_hands_it() {
    let sandbox = Sandbox::new("preview");
    let cache = sandbox.file("rows.jsonl", CACHED_ROWS);

    assert_eq!(preview(&cache, "1").expect("a preview"), "the second\nrow");
    assert_eq!(
        preview(&cache, "1\t").expect("a preview"),
        "the second\nrow"
    );
    assert_eq!(preview(&cache, "0").expect("no preview"), "");
    assert_eq!(preview(&cache, "9").expect("no row"), "");
}

#[test]
fn preview_of_a_missing_cache_fails() {
    let refusal = preview(Path::new("/nonexistent/rows.jsonl"), "0");

    assert!(
        matches!(refusal, Err(CommandError::Failed(_))),
        "{refusal:?}"
    );
}

#[test]
fn init_bash_binds_each_configured_picker() {
    let sandbox = Sandbox::new("init");
    let config = sandbox.file(
        "config.toml",
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = \"files.sh\"\n",
    );

    let script = init("bash", &config).expect("a bash script");

    assert!(script.starts_with("__herdr_picker() {\n"), "{script}");
    assert!(
        script.ends_with("}\nbind -x '\"\\C-tf\": __herdr_picker files'\n"),
        "{script}"
    );
}

#[test]
fn init_for_a_shell_it_does_not_know_is_a_usage_error() {
    let refusal = init("zsh", Path::new("/unused/config.toml"));

    assert!(matches!(refusal, Err(CommandError::Usage)), "{refusal:?}");
}
