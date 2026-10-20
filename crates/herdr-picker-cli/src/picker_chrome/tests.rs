use std::fs;
use std::path::PathBuf;

use herdr_picker_adapters::{picker_error_file, read_view};

use super::*;

const GROUPED_ROWS: &str = r#"{"label":"main","group":"remote","enter":{"insert":"main"}}
{"label":"dev","group":"Local","enter":{"insert":"dev"}}
{"label":"next","group":"remote","enter":{"insert":"next"}}
"#;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str, rows: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-cli-chrome-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        fs::write(directory.join("rows.jsonl"), rows).expect("a rows cache");
        Self(directory)
    }

    fn cache(&self) -> PathBuf {
        self.0.join("rows.jsonl")
    }

    fn press(&self, event: &[&str]) -> String {
        view_key(&self.cache(), event).expect("fzf actions")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn reload(cache: &Path) -> String {
    format!(
        "reload({} rows {})",
        shell_quote(
            &env::current_exe()
                .expect("the test binary")
                .to_string_lossy()
        ),
        shell_quote(&cache.to_string_lossy())
    )
}

#[test]
fn the_header_names_the_source_once_the_rows_carry_groups() {
    let grouped = Sandbox::new("header-grouped", GROUPED_ROWS);
    let plain = Sandbox::new(
        "header-plain",
        "{\"label\":\"a\",\"enter\":{\"insert\":\"a\"}}\n",
    );

    assert_eq!(
        picker_header(&grouped.cache(), "branches").expect("a header"),
        "Branches | Source: all"
    );
    assert_eq!(
        picker_header(&plain.cache(), "branches").expect("a header"),
        "Branches"
    );
}

#[test]
fn loading_grouped_rows_redraws_them_with_the_group_column() {
    let sandbox = Sandbox::new("load", GROUPED_ROWS);

    assert_eq!(sandbox.press(&["load"]), reload(&sandbox.cache()));
    assert_eq!(
        view_rows(&sandbox.cache()).expect("rows"),
        "0\t\x1b[2mremote\x1b[22m  main\n1\t\x1b[2mlocal \x1b[22m  dev\n2\t\x1b[2mremote\x1b[22m  next\n"
    );
    assert_eq!(sandbox.press(&["load"]), "");
}

#[test]
fn choosing_a_group_filters_the_list_and_names_it_in_the_header() {
    let sandbox = Sandbox::new("choose", GROUPED_ROWS);
    let cache = sandbox.cache();
    sandbox.press(&["load"]);

    assert_eq!(
        sandbox.press(&["choose"]),
        format!("{}+clear-query+hide-preview", reload(&cache))
    );
    assert_eq!(
        view_rows(&cache).expect("chooser rows"),
        "0\tall\n1\tlocal\n2\tremote\n"
    );
    assert_eq!(
        picker_header(&cache, "branches").expect("a header"),
        "Branches › Choose source"
    );

    assert_eq!(
        sandbox.press(&["enter", "2"]),
        format!("{}+clear-query+show-preview", reload(&cache))
    );
    assert_eq!(
        view_rows(&cache).expect("filtered rows"),
        "0\t\x1b[2mremote\x1b[22m  main\n2\t\x1b[2mremote\x1b[22m  next\n"
    );
    assert_eq!(
        picker_header(&cache, "branches").expect("a header"),
        "Branches | Source: remote"
    );
    assert_eq!(sandbox.press(&["enter", "2"]), "accept");
}

#[test]
fn back_from_the_chooser_keeps_the_group_and_back_from_the_list_closes() {
    let sandbox = Sandbox::new("back", GROUPED_ROWS);
    sandbox.press(&["load"]);
    sandbox.press(&["choose"]);
    sandbox.press(&["enter", "1"]);
    sandbox.press(&["choose"]);

    sandbox.press(&["back"]);

    assert_eq!(
        read_view(&sandbox.cache()),
        View::List {
            group: Some("local".to_string())
        }
    );
    assert_eq!(sandbox.press(&["back"]), "abort");
}

#[test]
fn a_run_error_reaches_the_footer_when_the_rows_finish_loading() {
    let sandbox = Sandbox::new("error", "");
    fs::write(
        picker_error_file(&sandbox.cache()),
        "line 1 is not a picker row: oops",
    )
    .expect("an error file");

    assert_eq!(
        sandbox.press(&["load"]),
        format!(
            "transform-footer({} footer {})",
            shell_quote(
                &env::current_exe()
                    .expect("the test binary")
                    .to_string_lossy()
            ),
            shell_quote(&sandbox.cache().to_string_lossy())
        )
    );
    assert_eq!(
        run_error_footer(&sandbox.cache(), Some(60), Theme::CatppuccinMocha),
        "\x1b[38;2;243;139;168mline 1 is not a picker\x1b[0m\n\x1b[38;2;243;139;168mrow: oops\x1b[0m"
    );
}

#[test]
fn an_unknown_view_event_is_a_usage_error() {
    let sandbox = Sandbox::new("usage", GROUPED_ROWS);

    let refusal = view_key(&sandbox.cache(), &["sideways"]);

    assert!(matches!(refusal, Err(CommandError::Usage)), "{refusal:?}");
}
