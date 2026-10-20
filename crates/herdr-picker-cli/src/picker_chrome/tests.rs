use std::fs;
use std::path::PathBuf;

use herdr_picker_adapters::{picker_error_file, write_view};
use herdr_picker_domain::ViewState;

use super::*;

const GROUPED_ROWS: &str = r#"{"label":"main","group":"remote","enter":{"insert":"main"},"keys":{"ctrl-o":{"open":"https://example.com/main"}}}
{"label":"dev","group":"Local","enter":{"insert":"dev"}}
"#;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str, rows: &str, view: View) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-cli-chrome-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        let sandbox = Self(directory);
        fs::write(sandbox.cache(), rows).expect("a rows cache");
        let state = ViewState {
            view,
            titles: vec!["Git Branches".to_string(), "Files".to_string()],
            description: "Every file".to_string(),
        };
        write_view(&sandbox.cache(), &state).expect("a view file");
        sandbox
    }

    fn cache(&self) -> PathBuf {
        self.0.join("rows.jsonl")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn list() -> View {
    View::List { group: None }
}

#[test]
fn the_header_is_the_trail_then_the_source_once_the_rows_carry_groups() {
    let grouped = Sandbox::new("header-grouped", GROUPED_ROWS, list());
    let plain = Sandbox::new(
        "header-plain",
        "{\"label\":\"a\",\"enter\":{\"insert\":\"a\"}}\n",
        list(),
    );

    assert_eq!(
        picker_header(&grouped.cache()).expect("a header"),
        "Git Branches › Files | Source: all"
    );
    assert_eq!(
        picker_header(&plain.cache()).expect("a header"),
        "Git Branches › Files"
    );
}

#[test]
fn the_footer_shows_the_highlighted_rows_keys_above_the_description() {
    let sandbox = Sandbox::new("footer", GROUPED_ROWS, list());

    assert_eq!(
        picker_footer(
            &sandbox.cache(),
            Some("0"),
            Some(200),
            Theme::CatppuccinMocha
        )
        .expect("a footer"),
        "enter insert · ctrl-o open · F1 help\nEvery file"
    );
    assert_eq!(
        picker_footer(&sandbox.cache(), None, None, Theme::CatppuccinMocha).expect("a footer"),
        "Every file"
    );
}

#[test]
fn the_chooser_footer_has_no_row_keys() {
    let sandbox = Sandbox::new(
        "footer-chooser",
        GROUPED_ROWS,
        View::Chooser { group: None },
    );

    assert_eq!(
        picker_footer(&sandbox.cache(), Some("0"), None, Theme::CatppuccinMocha).expect("a footer"),
        "Every file"
    );
}

#[test]
fn a_run_error_stays_in_the_footer_below_the_rows_keys() {
    let sandbox = Sandbox::new("error", "", list());
    fs::write(
        picker_error_file(&sandbox.cache()),
        "line 1 is not a picker row: oops",
    )
    .expect("an error file");

    assert_eq!(
        picker_footer(&sandbox.cache(), None, Some(60), Theme::CatppuccinMocha).expect("a footer"),
        "Every file\n\x1b[38;2;243;139;168mline 1 is not a picker\x1b[0m\n\x1b[38;2;243;139;168mrow: oops\x1b[0m"
    );
}

#[test]
fn help_lists_the_highlighted_rows_keys_then_the_general_ones_under_the_header() {
    let sandbox = Sandbox::new("help", GROUPED_ROWS, list());

    let page = picker_help(&sandbox.cache(), Some("0"), Theme::CatppuccinMocha).expect("help");

    let plain: String = page
        .split('\x1b')
        .map(|part| part.split_once('m').map_or(part, |(_, rest)| rest))
        .collect();
    assert_eq!(
        plain,
        "Git Branches › Files | Source: all › Help\n\n\
         enter       insert main\n\
         ctrl-o      open https://example.com/main\n\
         ctrl-space  choose a source\n\
         tab         mark a row; a key then acts on every marked row\n\
         esc         back to the previous step, or close on the first\n\
         F1          this help"
    );
}

#[test]
fn an_unknown_part_is_a_usage_error() {
    let sandbox = Sandbox::new("usage", GROUPED_ROWS, list());

    let refusal = answer("sideways", &sandbox.cache(), &[], Path::new("/unused"));

    assert!(matches!(refusal, Err(CommandError::Usage)), "{refusal:?}");
}
