use pwe_domain::Theme;

use super::*;

#[test]
fn an_error_footer_is_red_and_wrapped_on_words_to_the_width() {
    assert_eq!(
        error_footer(
            "line 2 is not a picker row: expected value",
            16,
            Theme::CatppuccinMocha
        ),
        "\x1b[38;2;243;139;168mline 2 is not a\x1b[0m\n\x1b[38;2;243;139;168mpicker row:\x1b[0m\n\x1b[38;2;243;139;168mexpected value\x1b[0m"
    );
}

#[test]
fn a_steps_footer_is_the_rows_keys_then_the_description_then_any_error_in_red() {
    assert_eq!(
        step_footer(
            &["enter insert".to_string(), "F1 help".to_string()],
            "Every file",
            "line 2 is bad",
            40,
            Theme::CatppuccinMocha
        ),
        "enter insert · F1 help\nEvery file\n\x1b[38;2;243;139;168mline 2 is bad\x1b[0m"
    );
    assert_eq!(
        step_footer(&[], "Every file", "", 40, Theme::CatppuccinMocha),
        "Every file"
    );
}

#[test]
fn the_rows_keys_wrap_between_entries_to_the_footers_width() {
    let hint = [
        "enter insert",
        "ctrl-enter edit",
        "alt-y copy and stay",
        "F1 help",
    ]
    .map(String::from);

    assert_eq!(
        step_footer(&hint, "", "", 32, Theme::CatppuccinMocha),
        "enter insert · ctrl-enter edit\nalt-y copy and stay · F1 help"
    );
}

#[test]
fn the_help_page_titles_itself_like_the_header_and_aligns_each_key() {
    let entries = [
        ("enter".to_string(), "insert main".to_string()),
        ("ctrl-space".to_string(), "choose a source".to_string()),
    ];

    assert_eq!(
        help_page("Git Branches", &entries, Theme::CatppuccinMocha),
        "\x1b[38;2;127;132;156mGit Branches › Help\x1b[0m\n\n\x1b[1;38;2;137;180;250menter     \x1b[0m  insert main\n\x1b[1;38;2;137;180;250mctrl-space\x1b[0m  choose a source"
    );
}
