use super::*;

fn words(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|word| word.to_string()).collect()
}

#[test]
fn the_preview_is_what_the_program_prints() {
    assert_eq!(command_preview(&words(&["printf", "a\\nb"])), "a\nb");
}

#[test]
fn a_failing_program_shows_what_it_said_on_stderr() {
    assert_eq!(
        command_preview(&words(&["sh", "-c", "echo out; echo oops >&2; exit 3"])),
        "out\noops\n"
    );
}

#[test]
fn a_missing_program_or_none_says_so() {
    assert!(
        command_preview(&words(&["/nonexistent/hwj-preview"]))
            .starts_with("Cannot run /nonexistent/hwj-preview: ")
    );
    assert_eq!(command_preview(&[]), "");
}
