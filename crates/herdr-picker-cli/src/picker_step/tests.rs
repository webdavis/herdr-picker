use std::env;
use std::ffi::OsStr;
use std::process::Command;

use herdr_picker_adapters::read_config;
use herdr_picker_domain::{Location, PickerCall, PickerRow};

use herdr_picker_protocol::{PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE};

use super::*;

mod send;

fn config(name: &str, body: &str) -> PluginConfig {
    let directory = env::temp_dir().join(format!("hwj-cli-step-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a config directory");
    let path = directory.join("config.toml");
    std::fs::write(&path, body).expect("a config");
    let config = read_config(&path).expect("a config");
    let _ = std::fs::remove_dir_all(&directory);
    config
}

fn words(command: &Command) -> Vec<String> {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|word| word.to_string_lossy().into_owned())
        .collect()
}

fn call(name: &str, args: &[&str], state: Option<&str>) -> Page {
    Page::Picker(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state: state.map(str::to_string),
        query: None,
    })
}

fn own() -> String {
    env::current_exe()
        .expect("the test binary")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn a_builtin_runs_herdr_picker_builtin_with_the_declared_then_the_given_args() {
    let config = config(
        "builtin",
        "[pickers.echo]\nkey = \"ctrl-t e\"\ndescription = \"Echo\"\nbuiltin = \"echo-args\"\nargs = [\"a\"]\n",
    );

    let StepCommand {
        command,
        description,
        ..
    } = step_command(&call("echo", &["b"], Some("{\"n\":1}")), None, &config).expect("a command");

    assert_eq!(
        words(&command),
        vec![
            own(),
            "builtin".into(),
            "echo-args".into(),
            "a".into(),
            "b".into()
        ]
    );
    assert_eq!(description, "Echo");
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        vec![
            (OsStr::new(PICKER_SOURCE_VARIABLE), None),
            (
                OsStr::new(PICKER_STATE_VARIABLE),
                Some(OsStr::new("{\"n\":1}"))
            )
        ]
    );
}

#[test]
fn a_program_runs_with_its_own_words_and_no_state_unless_given_one() {
    let config = config(
        "program",
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = [\"/bin/ls\", \"-a\"]\n",
    );

    let command = step_command(&call("files", &[], None), None, &config)
        .expect("a command")
        .command;

    assert_eq!(words(&command), vec!["/bin/ls", "-a"]);
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        vec![
            (OsStr::new(PICKER_SOURCE_VARIABLE), None),
            (OsStr::new(PICKER_STATE_VARIABLE), None)
        ]
    );
}

#[test]
fn a_confirm_step_runs_the_confirm_builtin_with_its_question_and_action() {
    let page = Page::Confirm {
        question: "Delete x?".to_string(),
        action: Action::Run("rm x".to_string()),
    };

    let StepCommand {
        command,
        description,
        ..
    } = step_command(&page, None, &config("confirm", "")).expect("a command");

    assert_eq!(
        words(&command),
        vec![
            own(),
            "builtin".into(),
            "confirm".into(),
            "Delete x?".into(),
            r#"{"run":"rm x"}"#.into()
        ]
    );
    assert_eq!(description, "");
}

#[test]
fn a_next_to_a_picker_the_config_lacks_is_refused() {
    let refusal = step_command(&call("ghost", &[], None), None, &config("ghost", "")).map(|_| ());

    assert_eq!(
        refusal.map_err(|failure| format!("{failure:?}")),
        Err("Failed(\"no picker is named ghost\")".to_string())
    );
}

#[test]
fn esc_goes_back_and_any_other_key_does_what_the_marked_rows_say() {
    let row = |path: &str| PickerRow {
        label: path.to_string(),
        preview: None,
        enter: Action::Edit(vec![Location {
            path: path.to_string(),
            line: None,
        }]),
        keys: Vec::new(),
        group: None,
    };
    let picked = |key: &str| Picked {
        query: String::new(),
        key: key.to_string(),
        rows: vec![row("a"), row("b")],
    };

    assert_eq!(
        chosen_action(&picked("esc"), None, Path::new("/q.json"), "Files"),
        Action::Back
    );
    assert_eq!(
        chosen_action(&picked("ctrl-o"), None, Path::new("/q.json"), "Files"),
        Action::Nothing
    );
    assert_eq!(
        chosen_action(&picked("enter"), None, Path::new("/q.json"), "Files"),
        Action::Edit(vec![
            Location {
                path: "a".to_string(),
                line: None
            },
            Location {
                path: "b".to_string(),
                line: None
            },
        ])
    );
}

#[test]
fn leaving_to_save_the_quickfix_list_opens_neovim_with_it() {
    let picked = Picked {
        query: String::new(),
        key: SAVE_QUICKFIX.to_string(),
        rows: Vec::new(),
    };

    assert_eq!(
        chosen_action(&picked, None, Path::new("/state/quickfix.json"), "Files"),
        Action::Run(
            "nvim -c 'call setqflist(json_decode(join(readfile('\\''/state/quickfix.json'\\''))))' -c copen"
                .to_string()
        )
    );
}

fn source_of(step: &StepCommand) -> Option<String> {
    step.command
        .get_envs()
        .find(|(variable, _)| *variable == OsStr::new(PICKER_SOURCE_VARIABLE))
        .and_then(|(_, value)| value)
        .map(|value| value.to_string_lossy().into_owned())
}

#[test]
fn a_builtin_with_sources_reads_its_first_source_unless_another_was_chosen() {
    let config = config("sources", "");

    let first = step_command(&call("files", &[], None), None, &config).expect("a command");
    let chosen = step_command(&call("files", &[], None), Some("home"), &config).expect("a command");

    assert_eq!(source_of(&first), Some("this folder".to_string()));
    assert_eq!(first.source, Some("this folder".to_string()));
    assert_eq!(
        first.sources,
        ["this folder", "hidden", "home", "whole disk", "recent"]
    );
    assert_eq!(source_of(&chosen), Some("home".to_string()));
    assert_eq!(chosen.source, Some("home".to_string()));
}

#[test]
fn a_source_picked_in_the_chooser_shows_the_step_again_from_that_source() {
    let picked = Picked {
        query: "hid".to_string(),
        key: SWITCH_SOURCE.to_string(),
        rows: Vec::new(),
    };

    assert_eq!(
        chosen_action(&picked, Some("hidden"), Path::new("/q.json"), "Files"),
        Action::Source("hidden".to_string())
    );
}

#[test]
fn a_step_builtin_the_config_never_names_runs_from_a_next_with_its_args() {
    let step = step_command(
        &call("rename-branch", &["/repo", "main"], None),
        None,
        &config("step-builtin", ""),
    )
    .expect("a command");

    assert_eq!(
        words(&step.command),
        vec![
            own(),
            "builtin".into(),
            "rename-branch".into(),
            "/repo".into(),
            "main".into()
        ]
    );
    assert!(step.sources.is_empty());
    assert!(step.live);
}

#[test]
fn a_step_builtin_with_sources_reads_its_first_unless_another_was_chosen() {
    let config = config("step-sources", "");
    let file = call("data-file", &["/a.json"], None);

    let first = step_command(&file, None, &config).expect("a command");
    let chosen = step_command(&file, Some("levels"), &config).expect("a command");

    assert_eq!(source_of(&first), Some("paths".to_string()));
    assert_eq!(
        first.sources,
        ["paths", "zoom", "keys", "levels", "shape", "tables"]
    );
    assert_eq!(source_of(&chosen), Some("levels".to_string()));
    assert_eq!(chosen.source, Some("levels".to_string()));
}

#[test]
fn a_step_is_live_when_its_builtin_matches_the_query_for_its_source() {
    let config = config("live", "");
    let live = |name: &str, source: Option<&str>| {
        step_command(&call(name, &[], None), source, &config)
            .expect("a command")
            .live
    };

    assert!(live("contents", None));
    assert!(live("history", Some("changes")));
    assert!(!live("history", None));
    assert!(!live("files", None));
}
