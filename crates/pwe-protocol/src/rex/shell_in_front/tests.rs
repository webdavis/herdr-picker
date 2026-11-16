use super::*;

fn process(child: u32, foreground: &str) -> String {
    format!(
        r#"{{"child":{{"pid":{child},"name":"zsh","argv0":"-zsh"}},"foreground":{foreground}}}"#
    )
}

#[test]
fn the_shell_rex_started_in_front_is_waiting() {
    let answer = process(
        3001,
        r#"{"pid":3001,"name":"zsh","argv0":"-zsh","cwd":"/work/project"}"#,
    );

    assert_eq!(
        rex_pane_front(&answer),
        Ok(PaneFront::TheShell {
            folder: Some("/work/project".to_string())
        })
    );
}

#[test]
fn another_program_in_front_is_not_the_shell() {
    let answer = process(3001, r#"{"pid":3002,"name":"vim","argv0":"vim"}"#);

    assert_eq!(rex_pane_front(&answer), Ok(PaneFront::AnotherProgram));
}

#[test]
fn a_pane_with_nothing_in_front_is_not_known_yet() {
    assert_eq!(
        rex_pane_front(&process(3001, "null")),
        Ok(PaneFront::NotKnown)
    );
}
