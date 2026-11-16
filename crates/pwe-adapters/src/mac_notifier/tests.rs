use pwe_application::{DesktopNotifier, DesktopNotifierError, DesktopRoute};

use super::MacNotifier;
use crate::fake_program::{Answer, FakePrograms};

const SENT: Answer<'static> = Answer {
    stdout: "",
    stderr: "",
    exit_code: 0,
};

fn with_programs(names: &[&str]) -> FakePrograms {
    names.iter().fold(FakePrograms::new(), |programs, name| {
        programs.answering(name, SENT)
    })
}

fn notify(
    programs: &FakePrograms,
    route: DesktopRoute,
    title: &str,
    text: &str,
) -> Result<(), DesktopNotifierError> {
    MacNotifier::new(route, Some(programs.path_variable())).notify(title, text)
}

#[test]
fn the_text_reaches_osascript_as_an_argument() {
    let programs = with_programs(&["osascript"]);

    notify(
        &programs,
        DesktopRoute::Auto,
        "pwe tests",
        "Tests failed: 3",
    )
    .expect("sent");

    let arguments = programs.arguments("osascript").expect("osascript ran");
    assert_eq!(
        arguments[arguments.len() - 3..],
        ["--", "pwe tests", "Tests failed: 3"]
    );
    assert!(
        arguments[..arguments.len() - 3]
            .iter()
            .all(|argument| !argument.contains("Tests failed") && !argument.contains("pwe tests")),
        "{arguments:?}"
    );
}

#[test]
fn a_text_holding_quotes_and_a_backslash_arrives_unchanged() {
    let programs = with_programs(&["osascript"]);
    let text = r#"done" & (do shell script "echo \ broken") & ""#;

    notify(&programs, DesktopRoute::Osascript, "it's \"quoted\"", text).expect("sent");

    let arguments = programs.arguments("osascript").expect("osascript ran");
    assert_eq!(arguments[arguments.len() - 2..], ["it's \"quoted\"", text]);
}

#[test]
fn terminal_notifier_is_chosen_when_it_is_on_path() {
    let programs = with_programs(&["osascript", "terminal-notifier"]);

    notify(&programs, DesktopRoute::Auto, "pwe tests", "Tests passed").expect("sent");

    assert_eq!(
        programs.arguments("terminal-notifier"),
        Some(
            ["-title", "pwe tests", "-message", "Tests passed"]
                .map(String::from)
                .to_vec()
        )
    );
    assert_eq!(programs.arguments("osascript"), None);
}

#[test]
fn a_chosen_osascript_is_used_while_terminal_notifier_is_on_path() {
    let programs = with_programs(&["osascript", "terminal-notifier"]);

    notify(&programs, DesktopRoute::Osascript, "pwe", "text").expect("sent");

    assert!(programs.arguments("osascript").is_some());
    assert_eq!(programs.arguments("terminal-notifier"), None);
}

#[test]
fn off_sends_nothing() {
    let programs = with_programs(&["osascript", "terminal-notifier"]);

    assert_eq!(notify(&programs, DesktopRoute::Off, "pwe", "text"), Ok(()));

    assert_eq!(
        (
            programs.arguments("osascript"),
            programs.arguments("terminal-notifier")
        ),
        (None, None)
    );
}

#[test]
fn with_neither_program_it_says_so_and_sends_nothing() {
    let programs = with_programs(&[]);

    let refusal =
        notify(&programs, DesktopRoute::Auto, "pwe", "text").expect_err("nothing to send with");

    assert_eq!(refusal, DesktopNotifierError::NoProgram);
    let words = refusal.to_string();
    assert!(
        words.contains("terminal-notifier") && words.contains("osascript"),
        "{words}"
    );
}

#[test]
fn a_chosen_program_that_is_missing_is_named() {
    let programs = with_programs(&["osascript"]);

    assert_eq!(
        notify(&programs, DesktopRoute::TerminalNotifier, "pwe", "text"),
        Err(DesktopNotifierError::ProgramMissing(
            "terminal-notifier".to_string()
        ))
    );
}

#[test]
fn a_failing_program_keeps_its_reason() {
    let programs = FakePrograms::new().answering(
        "osascript",
        Answer {
            stdout: "",
            stderr: "execution error: example failure (-1)",
            exit_code: 1,
        },
    );

    let Err(DesktopNotifierError::Refused { program, detail }) =
        notify(&programs, DesktopRoute::Auto, "pwe", "text")
    else {
        panic!("a failing osascript is refused");
    };
    assert_eq!(program, "osascript");
    assert!(detail.contains("example failure"), "{detail}");
}
