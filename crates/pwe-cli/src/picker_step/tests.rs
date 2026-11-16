use std::env;
use std::ffi::OsStr;
use std::process::Command;

use pwe_adapters::read_config;
use pwe_domain::PickerCall;

use pwe_protocol::{PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE};

use super::*;

mod chosen;
mod finder;
mod scripted;
mod send;
mod sources;

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
fn a_builtin_runs_pwe_builtin_with_the_declared_then_the_given_args() {
    let config = config(
        "builtin",
        "[pickers.echo]\nkey = \"ctrl-t e\"\ndescription = \"Echo\"\nbuiltin = \"echo-args\"\nargs = [\"a\"]\n",
    );

    let StepCommand {
        command,
        description,
        ..
    } = step_command(&call("echo", &["b"], Some("{\"n\":1}")), None, &config, &[])
        .expect("a command");

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

    let command = step_command(&call("files", &[], None), None, &config, &[])
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
    } = step_command(&page, None, &config("confirm", ""), &[]).expect("a command");

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
    let refusal =
        step_command(&call("ghost", &[], None), None, &config("ghost", ""), &[]).map(|_| ());

    assert_eq!(
        refusal.map_err(|failure| format!("{failure:?}")),
        Err("Failed(\"no picker is named ghost\")".to_string())
    );
}

#[test]
fn a_step_builtin_the_config_never_names_runs_from_a_next_with_its_args() {
    let step = step_command(
        &call("rename-branch", &["/repo", "main"], None),
        None,
        &config("step-builtin", ""),
        &[],
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
fn a_step_is_live_when_its_builtin_matches_the_query_for_its_source() {
    let config = config("live", "");
    let live = |name: &str, source: Option<&str>| {
        step_command(&call(name, &[], None), source, &config, &[])
            .expect("a command")
            .live
    };

    assert!(live("contents", None));
    assert!(live("history", Some("changes")));
    assert!(!live("history", None));
    assert!(!live("files", None));
}
