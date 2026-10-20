use herdr_picker_domain::Preview;

use super::*;

mod actions;

fn row(label: &str, preview: Option<&str>, enter: Action) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: preview.map(|text| Preview::Text(text.to_string())),
        enter,
        keys: Vec::new(),
        group: None,
    }
}

#[test]
fn a_row_carries_a_label_an_optional_preview_and_one_enter_action() {
    assert_eq!(
        parse_row(
            r#"{"label":"main","preview":"3 commits ahead","enter":{"insert":"main"}}"#,
            1
        ),
        Ok(row(
            "main",
            Some("3 commits ahead"),
            Action::Insert("main".to_string())
        ))
    );
    assert_eq!(
        parse_row(r#"{"label":"sha","enter":{"copy":"abc123"}}"#, 1),
        Ok(row("sha", None, Action::Copy("abc123".to_string())))
    );
    assert_eq!(
        parse_row(r#"{"label":"repo","enter":{"cd":"/opt/repo"}}"#, 1),
        Ok(row("repo", None, Action::Cd("/opt/repo".to_string())))
    );
    assert_eq!(
        parse_row(r#"{"label":"test","enter":{"run":"just test"}}"#, 1),
        Ok(row("test", None, Action::Run("just test".to_string())))
    );
}

#[test]
fn a_label_keeps_its_ansi_colors() {
    let parsed = parse_row(
        r#"{"label":"\u001b[32mmain\u001b[0m","enter":{"insert":"main"}}"#,
        1,
    )
    .expect("a row");

    assert_eq!(parsed.label, "\x1b[32mmain\x1b[0m");
}

#[test]
fn a_bad_line_is_refused_naming_its_line_number() {
    for bad in [
        "not json",
        r#"{"label":"main"}"#,
        r#"{"label":"main","enter":{"launch":"main"}}"#,
        r#"{"label":"main","enter":{"insert":"main"},"extra":1}"#,
    ] {
        let refusal = parse_row(bad, 7).expect_err("a bad line is refused");
        assert_eq!(refusal.line, 7, "{bad}");
        assert!(
            refusal
                .to_string()
                .starts_with("line 7 is not a picker row: "),
            "{refusal}"
        );
    }
}

#[test]
fn a_label_that_spans_lines_is_refused_since_the_finder_reads_one_row_per_line() {
    let refusal = parse_row(r#"{"label":"two\nlines","enter":{"insert":"x"}}"#, 2)
        .expect_err("a multi-line label is refused");

    assert_eq!(
        refusal.to_string(),
        "line 2 is not a picker row: the label spans more than one line"
    );
}

#[test]
fn a_row_may_name_its_group_which_is_always_lowercase() {
    let parsed = parse_row(
        r#"{"label":"main","group":"Remote Branches","enter":{"insert":"main"}}"#,
        1,
    )
    .expect("a row");

    assert_eq!(parsed.group, Some("remote branches".to_string()));
}

#[test]
fn a_group_with_control_characters_is_refused() {
    let refusal = parse_row(
        r#"{"label":"main","group":"a\tb","enter":{"insert":"main"}}"#,
        3,
    )
    .expect_err("a group with a tab is refused");

    assert_eq!(
        refusal.to_string(),
        "line 3 is not a picker row: the group holds a control character"
    );
}

#[test]
fn a_preview_may_name_a_path_to_draw_once_the_row_is_highlighted() {
    let line = r#"{"label":"notes.txt","preview":{"path":"/work/notes.txt"},"enter":{"insert":"notes.txt"}}"#;

    let parsed = parse_row(line, 1).expect("a row");

    assert_eq!(
        parsed.preview,
        Some(Preview::Path("/work/notes.txt".to_string()))
    );
    assert_eq!(encode_row(&parsed), line);
}
