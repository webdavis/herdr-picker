use std::env;
use std::fs;
use std::path::PathBuf;

use herdr_picker_adapters::{picker_error_file, write_view};
use herdr_picker_domain::View;
use herdr_picker_protocol::shell_quote;

use super::*;

const GROUPED_ROWS: &str = r#"{"label":"main","group":"remote","enter":{"insert":"main"},"keys":{"ctrl-o":{"cd":"/opt/main"}}}
{"label":"dev","group":"Local","enter":{"insert":"dev"},"keys":{"ctrl-o":{"cd":"/opt/dev"}}}
{"label":"next","group":"remote","enter":{"insert":"next"}}
"#;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str, rows: &str) -> Self {
        let directory = env::temp_dir().join(format!("hwj-cli-keys-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        fs::write(directory.join("rows.jsonl"), rows).expect("a rows cache");
        Self(directory)
    }

    fn cache(&self) -> PathBuf {
        self.0.join("rows.jsonl")
    }

    fn press(&self, event: &[&str]) -> String {
        view_key(&self.cache(), event, Theme::CatppuccinMocha).expect("fzf actions")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn helper_call(cache: &Path, subcommand: &str) -> String {
    format!(
        "{} {subcommand} {}",
        shell_quote(
            &env::current_exe()
                .expect("the test binary")
                .to_string_lossy()
        ),
        shell_quote(&cache.to_string_lossy())
    )
}

fn red(text: &str) -> String {
    format!("change-footer:\x1b[38;2;243;139;168m{text}\x1b[0m")
}

#[test]
fn loading_grouped_rows_redraws_them_once() {
    let sandbox = Sandbox::new("load", GROUPED_ROWS);

    assert_eq!(
        sandbox.press(&["load"]),
        format!("reload({})", helper_call(&sandbox.cache(), "rows"))
    );
    assert_eq!(sandbox.press(&["load"]), "");
}

#[test]
fn enter_in_the_chooser_picks_a_group_and_enter_in_the_list_leaves_with_the_key() {
    let sandbox = Sandbox::new("choose", GROUPED_ROWS);
    let reload = format!("reload({})", helper_call(&sandbox.cache(), "rows"));
    sandbox.press(&["load"]);

    assert_eq!(
        sandbox.press(&["choose"]),
        format!("{reload}+clear-query+hide-preview")
    );
    assert_eq!(sandbox.press(&["key", "ctrl-o", "1"]), "");
    assert_eq!(
        sandbox.press(&["key", "enter", "2"]),
        format!("{reload}+clear-query+show-preview")
    );
    assert_eq!(
        read_view(&sandbox.cache()).view,
        View::List {
            group: Some("remote".to_string())
        }
    );
    assert_eq!(sandbox.press(&["key", "enter", "2"]), "print(enter)+accept");
}

#[test]
fn a_rows_own_key_leaves_and_a_key_it_does_not_claim_does_nothing() {
    let sandbox = Sandbox::new("keys", GROUPED_ROWS);

    assert_eq!(
        sandbox.press(&["key", "ctrl-o", "0"]),
        "print(ctrl-o)+accept"
    );
    assert_eq!(sandbox.press(&["key", "alt-y", "0"]), "");
    assert_eq!(sandbox.press(&["key", "enter", ""]), "");
}

#[test]
fn a_key_no_row_claims_keeps_what_fzf_does_with_it() {
    let sandbox = Sandbox::new("defaults", GROUPED_ROWS);

    assert_eq!(sandbox.press(&["key", "ctrl-e", "0"]), "end-of-line");
    assert_eq!(sandbox.press(&["key", "alt-b", "2"]), "backward-word");
    assert_eq!(sandbox.press(&["key", "ctrl-a", ""]), "beginning-of-line");
    sandbox.press(&["load"]);
    sandbox.press(&["choose"]);
    assert_eq!(sandbox.press(&["key", "ctrl-w", "0"]), "unix-word-rubout");
}

#[test]
fn a_row_claiming_an_editing_key_takes_it_over() {
    let rows = "{\"label\":\"main\",\"enter\":{\"insert\":\"main\"},\"keys\":{\"ctrl-e\":{\"run\":\"git branch -m main\"}}}\n";
    let sandbox = Sandbox::new("claimed", rows);

    assert_eq!(
        sandbox.press(&["key", "ctrl-e", "0"]),
        "print(ctrl-e)+accept"
    );
}

#[test]
fn marked_rows_that_lead_one_place_ask_for_one_in_the_footer() {
    let sandbox = Sandbox::new("multi", GROUPED_ROWS);

    assert_eq!(
        sandbox.press(&["key", "ctrl-o", "0", "1"]),
        red("Select one row to change into")
    );
    assert_eq!(
        sandbox.press(&["key", "ctrl-o", "0", "2"]),
        red("The marked rows do different things on ctrl-o")
    );
    assert_eq!(
        sandbox.press(&["key", "enter", "0 2"]),
        "print(enter)+accept"
    );
}

#[test]
fn back_from_the_chooser_keeps_the_group_and_back_from_the_list_leaves_with_esc() {
    let sandbox = Sandbox::new("back", GROUPED_ROWS);
    sandbox.press(&["load"]);
    sandbox.press(&["choose"]);
    sandbox.press(&["key", "enter", "1"]);
    sandbox.press(&["choose"]);

    sandbox.press(&["back"]);

    assert_eq!(
        read_view(&sandbox.cache()).view,
        View::List {
            group: Some("local".to_string())
        }
    );
    assert_eq!(sandbox.press(&["back"]), "print(esc)+accept");
}

#[test]
fn a_step_returned_to_filters_to_its_group_when_its_rows_load() {
    let sandbox = Sandbox::new("return", GROUPED_ROWS);
    let state = ViewState {
        view: View::Streamed {
            group: Some("local".to_string()),
        },
        titles: Vec::new(),
        description: String::new(),
        source: None,
        sources: Vec::new(),
    };
    write_view(&sandbox.cache(), &state).expect("a view file");

    sandbox.press(&["load"]);

    assert_eq!(
        read_view(&sandbox.cache()).view,
        View::List {
            group: Some("local".to_string())
        }
    );
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
            "transform-footer({})",
            helper_call(&sandbox.cache(), "footer")
        )
    );
}

#[test]
fn an_unknown_view_event_or_key_is_a_usage_error() {
    let sandbox = Sandbox::new("usage", GROUPED_ROWS);

    for event in [&["sideways"][..], &["key", "ctrl-q", "0"]] {
        let refusal = view_key(&sandbox.cache(), event, Theme::CatppuccinMocha);
        assert!(matches!(refusal, Err(CommandError::Usage)), "{refusal:?}");
    }
}

#[test]
fn picking_a_source_in_the_chooser_records_it_and_leaves_to_read_it() {
    let sandbox = Sandbox::new("switch", "{\"label\":\"a\",\"enter\":{\"insert\":\"a\"}}\n");
    let state = ViewState {
        view: View::Streamed { group: None },
        titles: vec!["Files".to_string()],
        description: String::new(),
        source: Some("this folder".to_string()),
        sources: vec!["this folder".to_string(), "hidden".to_string()],
    };
    write_view(&sandbox.cache(), &state).expect("a view file");

    assert_eq!(
        sandbox.press(&["choose"]),
        format!(
            "reload({})+clear-query+hide-preview",
            helper_call(&sandbox.cache(), "rows")
        )
    );
    assert_eq!(
        sandbox.press(&["key", "enter", "1"]),
        "print(source)+accept"
    );
    assert_eq!(
        read_view(&sandbox.cache()).source,
        Some("hidden".to_string())
    );
}
