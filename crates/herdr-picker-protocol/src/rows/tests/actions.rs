use herdr_picker_domain::{Key, Location, PickerCall};

use super::*;

fn enter_of(line: &str) -> Result<Action, String> {
    parse_row(line, 1)
        .map(|row| row.enter)
        .map_err(|refusal| refusal.reason)
}

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

#[test]
fn every_action_reads_from_its_json_form() {
    let cases = [
        (r#""reload""#, Action::Reload),
        (r#""none""#, Action::Nothing),
        (r#""back""#, Action::Back),
        (
            r#"{"copy_stay":"abc"}"#,
            Action::CopyStay("abc".to_string()),
        ),
        (
            r#"{"open":"https://example.com"}"#,
            Action::Open("https://example.com".to_string()),
        ),
        (
            r#"{"edit":{"path":"src/a.rs","line":7}}"#,
            Action::Edit(vec![Location {
                path: "src/a.rs".to_string(),
                line: Some(7),
            }]),
        ),
        (
            r#"{"next":{"picker":"files","args":["--hidden"],"state":{"root":"/tmp"}}}"#,
            Action::Next(PickerCall {
                name: "files".to_string(),
                args: vec!["--hidden".to_string()],
                state: Some(r#"{"root":"/tmp"}"#.to_string()),
                query: None,
            }),
        ),
        (
            r#"{"next":{"picker":"rename-branch","args":["main"],"query":"main"}}"#,
            Action::Next(PickerCall {
                name: "rename-branch".to_string(),
                args: vec!["main".to_string()],
                state: None,
                query: Some("main".to_string()),
            }),
        ),
        (
            r#"{"state":{"regex":true}}"#,
            Action::State(r#"{"regex":true}"#.to_string()),
        ),
        (
            r#"{"confirm":{"question":"Delete x?","action":{"run":"rm x"}}}"#,
            Action::Confirm {
                question: "Delete x?".to_string(),
                action: Box::new(Action::Run("rm x".to_string())),
            },
        ),
    ];
    for (action, expected) in cases {
        let line = format!(r#"{{"label":"x","enter":{action}}}"#);
        assert_eq!(enter_of(&line), Ok(expected), "{action}");
    }
}

#[test]
fn a_list_inserts_as_shell_words_and_copies_one_per_line_while_a_text_stays_as_it_is() {
    assert_eq!(
        enter_of(r#"{"label":"x","enter":{"insert":["my file.txt","b.rs"]}}"#),
        Ok(Action::Insert("'my file.txt' b.rs".to_string()))
    );
    assert_eq!(
        enter_of(r#"{"label":"x","enter":{"insert":"git log -1"}}"#),
        Ok(Action::Insert("git log -1".to_string()))
    );
    assert_eq!(
        enter_of(r#"{"label":"x","enter":{"copy":["a","b"]}}"#),
        Ok(Action::Copy("a\nb".to_string()))
    );
}

#[test]
fn bare_insert_and_copy_use_the_rows_value() {
    let row = parse_row(
        r#"{"label":"x","value":["a b","c"],"enter":"insert","keys":{"alt-y":"copy","alt-c":"copy_stay"}}"#,
        1,
    )
    .expect("a row");

    assert_eq!(row.enter, Action::Insert("'a b' c".to_string()));
    assert_eq!(
        row.keys,
        vec![
            (key("alt-c"), Action::CopyStay("a b\nc".to_string())),
            (key("alt-y"), Action::Copy("a b\nc".to_string())),
        ]
    );
    assert_eq!(
        enter_of(r#"{"label":"x","enter":"copy"}"#),
        Err("an action uses the row's value, but the row has none".to_string())
    );
}

#[test]
fn keys_map_fzf_key_names_to_actions_and_ctrl_enter_is_alt_enter() {
    let row = parse_row(
        r#"{"label":"x","enter":"none","keys":{"ctrl-enter":{"run":"make"},"ctrl-o":{"open":"x"}}}"#,
        1,
    )
    .expect("a row");

    assert_eq!(
        row.keys,
        vec![
            (key("alt-enter"), Action::Run("make".to_string())),
            (key("ctrl-o"), Action::Open("x".to_string())),
        ]
    );
}

#[test]
fn a_key_a_row_cannot_claim_is_refused() {
    let refusal = |keys: &str| {
        enter_of(&format!(r#"{{"label":"x","enter":"none","keys":{keys}}}"#)).expect_err("refused")
    };

    assert_eq!(
        refusal(r#"{"ctrl-a":"none"}"#),
        "ctrl-a is not a key a row can claim"
    );
    assert_eq!(
        refusal(r#"{"enter":"none"}"#),
        "enter belongs in the row's enter field"
    );
    assert_eq!(
        refusal(r#"{"alt-enter":"none","ctrl-enter":"none"}"#),
        "ctrl-enter is a key the row already claims"
    );
}

#[test]
fn an_encoded_row_reads_back_as_the_same_row() {
    let line = r#"{"label":"x","preview":"p","group":"local","enter":{"edit":[{"path":"a"},{"path":"b","line":2}]},"keys":{"alt-y":{"copy":"a"},"ctrl-o":{"next":{"picker":"log","state":[1]}},"f2":{"confirm":{"question":"Sure?","action":"reload"}}}}"#;
    let row = parse_row(line, 1).expect("a row");

    assert_eq!(parse_row(&encode_row(&row), 1), Ok(row));
}

#[test]
fn an_action_alone_survives_its_round_trip() {
    let action = Action::Confirm {
        question: "Delete x?".to_string(),
        action: Box::new(Action::Cd("/tmp".to_string())),
    };

    assert_eq!(
        encode_action(&action),
        r#"{"confirm":{"question":"Delete x?","action":{"cd":"/tmp"}}}"#
    );
    assert_eq!(parse_action(&encode_action(&action)), Ok(action));
    assert!(parse_action("\"sideways\"").is_err());
}

#[test]
fn a_source_action_names_the_source_to_show_and_survives_encoding() {
    let line = r#"{"label":"a","enter":"none","keys":{"alt-.":{"source":"hidden"}}}"#;

    let parsed = parse_row(line, 1).expect("a row");

    assert_eq!(
        parsed.action(&Key::parse("alt-.").expect("a key")),
        Some(&Action::Source("hidden".to_string()))
    );
    assert_eq!(encode_row(&parsed), line);
}

#[test]
fn a_next_with_a_query_and_a_state_switch_write_back_as_they_read() {
    for line in [
        r#"{"label":"x","enter":{"next":{"picker":"rename-branch","args":["main"],"query":"main"}}}"#,
        r#"{"label":"x","enter":{"state":{"regex":true}}}"#,
    ] {
        let row = parse_row(line, 1).expect("a row");
        assert_eq!(encode_row(&row), line);
    }
}
