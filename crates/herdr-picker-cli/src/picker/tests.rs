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
    let in_vi_command: Vec<&str> = script
        .lines()
        .filter(|line| line.starts_with("bind -m vi-command "))
        .collect();
    assert_eq!(
        in_vi_command,
        [
            "bind -m vi-command -x '\"\\C-tf\": __herdr_picker files'",
            "bind -m vi-command -x '\"\\C-td\": __herdr_picker folders'",
            "bind -m vi-command -x '\"\\C-t\\C-b\": __herdr_picker bookmark'",
            "bind -m vi-command -x '\"\\C-gs\": __herdr_picker status'",
            "bind -m vi-command -x '\"\\C-gl\": __herdr_picker history'",
            "bind -m vi-command -x '\"\\C-gb\": __herdr_picker branches'",
            "bind -m vi-command -x '\"\\C-g/\": __herdr_picker contents'",
            "bind -m vi-command -x '\"\\C-gv\": __herdr_picker git-config'",
            "bind -m vi-command -x '\"\\C-tj\": __herdr_picker recipes'",
            "bind -m vi-command -x '\"\\C-tt\": __herdr_picker tests'",
            "bind -m vi-command -x '\"\\C-tp\": __herdr_picker processes'",
            "bind -m vi-command -x '\"\\C-tn\": __herdr_picker network'",
            "bind -m vi-command -x '\"\\C-ti\": __herdr_picker data'",
            "bind -m vi-command -x '\"\\C-ta\": __herdr_picker agents'",
            "bind -m vi-command -x '\"\\C-t;\": __herdr_picker pickers'",
            "bind -m vi-command -x '\"\\C-tm\": __herdr_picker apple'",
            "bind -m vi-command -x '\"\\C-tw\": __herdr_picker tabs'",
        ]
    );
    assert_eq!(script.matches("bind -m emacs-standard -x ").count(), 17);
    assert_eq!(script.matches("bind -m vi-insert -x ").count(), 17);
}

#[test]
fn init_bash_without_a_config_file_binds_the_builtin_pickers() {
    let sandbox = Sandbox::new("init-defaults");

    let script = init("bash", &sandbox.0.join("config.toml")).expect("a bash script");

    assert!(
        script.contains("bind -m vi-insert -x '\"\\C-tf\": __herdr_picker files'\n"),
        "{script}"
    );
}

#[test]
fn a_preview_naming_a_path_draws_that_path() {
    let sandbox = Sandbox::new("preview-path");
    let binary = sandbox.0.join("image.png");
    fs::write(&binary, [0, 1, 2]).expect("a file");
    let cache = sandbox.file(
        "rows.jsonl",
        &format!(
            "{{\"label\":\"image.png\",\"preview\":{{\"path\":\"{}\"}},\"enter\":\"none\"}}\n",
            binary.display()
        ),
    );

    assert_eq!(
        preview(&cache, "0").expect("a preview"),
        "Binary file, 3 bytes"
    );
}

#[test]
fn init_for_a_shell_it_does_not_know_is_a_usage_error() {
    let refusal = init("zsh", Path::new("/unused/config.toml"));

    assert!(matches!(refusal, Err(CommandError::Usage)), "{refusal:?}");
}
