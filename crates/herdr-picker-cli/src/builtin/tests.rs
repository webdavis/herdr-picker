use super::*;

#[test]
fn echo_args_prints_one_row_per_argument_that_inserts_it_as_a_shell_word() {
    assert_eq!(
        rows("echo-args", &["alpha", "my file"]).expect("rows"),
        concat!(
            r#"{"label":"alpha","enter":{"insert":"alpha"}}"#,
            "\n",
            r#"{"label":"my file","enter":{"insert":"'my file'"}}"#,
            "\n"
        )
    );
}

#[test]
fn confirm_prints_yes_running_the_action_and_no_going_back() {
    assert_eq!(
        rows("confirm", &["Delete x?", r#"{"run":"rm x"}"#]).expect("rows"),
        concat!(
            r#"{"label":"Yes","preview":"Delete x?","enter":{"run":"rm x"}}"#,
            "\n",
            r#"{"label":"No","preview":"Delete x?","enter":"back"}"#,
            "\n"
        )
    );
}

#[test]
fn an_unknown_builtin_or_a_bad_confirm_is_refused() {
    for (name, arguments) in [
        ("nope", &[][..]),
        ("confirm", &["Sure?", "\"sideways\""][..]),
    ] {
        let refusal = rows(name, arguments);
        assert!(
            matches!(refusal, Err(CommandError::Failed(_))),
            "{refusal:?}"
        );
    }
}
