use herdr_picker_domain::{Location, PickerCall};

use super::*;

const SHELL: Shell = Shell {
    editor: "nvim",
    cwd: "/work/repo",
    opener: "open",
};

fn edit(locations: &[(&str, Option<u32>)]) -> Action {
    Action::Edit(
        locations
            .iter()
            .map(|(path, line)| Location {
                path: path.to_string(),
                line: *line,
            })
            .collect(),
    )
}

fn line(action: &Action) -> Option<String> {
    instruction(action, &SHELL).expect("an instruction")
}

#[test]
fn words_split_like_the_shell_reads_them() {
    assert_eq!(
        split_words(r#"code  --wait 'my dir' "a \"b\"" c\ d ''"#),
        Ok(vec!["code", "--wait", "my dir", "a \"b\"", "c d", ""]
            .into_iter()
            .map(String::from)
            .collect())
    );
    assert_eq!(split_words("  "), Ok(Vec::new()));
    assert_eq!(split_words("vim 'oops"), Err(UnclosedQuote));
}

#[test]
fn a_word_is_quoted_only_when_the_shell_would_read_it_otherwise() {
    assert_eq!(shell_word("src/main.rs"), "src/main.rs");
    assert_eq!(shell_word("my file"), "'my file'");
    assert_eq!(shell_word("it's"), r"'it'\''s'");
    assert_eq!(shell_word(""), "''");
    assert_eq!(shell_quote("/tmp/my dir/it's"), r"'/tmp/my dir/it'\''s'");
}

#[test]
fn insert_cd_and_run_become_one_instruction_line_for_the_shell() {
    assert_eq!(
        line(&Action::Insert("git switch main".to_string())),
        Some("insert\tgit switch main".to_string())
    );
    assert_eq!(
        line(&Action::Cd("/opt/repo".to_string())),
        Some("cd\t/opt/repo".to_string())
    );
    assert_eq!(
        line(&Action::Run("just test".to_string())),
        Some("run\tjust test".to_string())
    );
}

#[test]
fn edit_and_open_come_back_as_commands_to_run_so_they_land_in_history() {
    assert_eq!(
        line(&edit(&[("/work/repo/src/main.rs", Some(12))])),
        Some("run\tnvim +12 src/main.rs".to_string())
    );
    assert_eq!(
        line(&Action::Open("https://example.com/a b".to_string())),
        Some("run\topen 'https://example.com/a b'".to_string())
    );
}

#[test]
fn edit_opens_every_path_once_relative_under_the_cwd_and_absolute_elsewhere() {
    assert_eq!(
        line(&edit(&[
            ("/work/repo/a.rs", None),
            ("/etc/hosts", Some(2)),
            ("/work/repo/a.rs", None),
            ("/work/repo", None),
            ("my notes.md", None),
        ])),
        Some("run\tnvim a.rs /etc/hosts . 'my notes.md'".to_string())
    );
}

#[test]
fn a_path_starting_with_a_dash_or_plus_gets_dot_slash_so_the_editor_reads_a_file() {
    assert_eq!(
        line(&edit(&[("/work/repo/-rf", None), ("+x", None)])),
        Some("run\tnvim ./-rf ./+x".to_string())
    );
}

#[test]
fn the_editor_may_carry_its_own_arguments() {
    let shell = Shell {
        editor: "'/Applications/My Editor/bin/code' --wait",
        ..SHELL
    };

    assert_eq!(
        instruction(&edit(&[("/work/repo/a.rs", Some(3))]), &shell),
        Ok(Some(
            "run\t'/Applications/My Editor/bin/code' --wait +3 a.rs".to_string()
        ))
    );
    assert_eq!(
        instruction(
            &edit(&[("a.rs", None)]),
            &Shell {
                editor: "vim 'x",
                ..SHELL
            }
        ),
        Err(UnclosedQuote)
    );
}

#[test]
fn actions_kept_inside_the_picker_give_the_shell_nothing() {
    for action in [
        Action::Copy("abc".to_string()),
        Action::CopyStay("abc".to_string()),
        Action::Next(PickerCall {
            name: "files".to_string(),
            args: Vec::new(),
            state: None,
        }),
        Action::Reload,
        Action::Back,
        Action::Nothing,
    ] {
        assert_eq!(line(&action), None, "{action:?}");
    }
}

#[test]
fn cd_is_relative_under_the_cwd_and_absolute_elsewhere() {
    let cd = |path: &str| line(&Action::Cd(path.to_string()));

    assert_eq!(cd("/work/repo/src/bin"), Some("cd\tsrc/bin".to_string()));
    assert_eq!(cd("/work/repo"), Some("cd\t.".to_string()));
    assert_eq!(cd("/work/other"), Some("cd\t/work/other".to_string()));
}

#[test]
fn a_source_action_stays_inside_the_picker() {
    assert_eq!(line(&Action::Source("hidden".to_string())), None);
}
