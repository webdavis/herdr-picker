use std::ffi::OsStr;

use herdr_picker_adapters::read_config;
use herdr_picker_domain::{Location, PickerCall, PickerRow};

use super::*;

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

fn call(name: &str, args: &[&str], state: Option<&str>) -> Source {
    Source::Picker(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state: state.map(str::to_string),
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

    let (command, description) =
        step_command(&call("echo", &["b"], Some("{\"n\":1}")), &config).expect("a command");

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
        vec![(
            OsStr::new(PICKER_STATE_VARIABLE),
            Some(OsStr::new("{\"n\":1}"))
        )]
    );
}

#[test]
fn a_program_runs_with_its_own_words_and_no_state_unless_given_one() {
    let config = config(
        "program",
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = [\"/bin/ls\", \"-a\"]\n",
    );

    let (command, _) = step_command(&call("files", &[], None), &config).expect("a command");

    assert_eq!(words(&command), vec!["/bin/ls", "-a"]);
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        vec![(OsStr::new(PICKER_STATE_VARIABLE), None)]
    );
}

#[test]
fn a_confirm_step_runs_the_confirm_builtin_with_its_question_and_action() {
    let source = Source::Confirm {
        question: "Delete x?".to_string(),
        action: Action::Run("rm x".to_string()),
    };

    let (command, description) = step_command(&source, &config("confirm", "")).expect("a command");

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
    let refusal = step_command(&call("ghost", &[], None), &config("ghost", "")).map(|_| ());

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

    assert_eq!(chosen_action(&picked("esc")), Action::Back);
    assert_eq!(chosen_action(&picked("ctrl-o")), Action::Nothing);
    assert_eq!(
        chosen_action(&picked("enter")),
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
