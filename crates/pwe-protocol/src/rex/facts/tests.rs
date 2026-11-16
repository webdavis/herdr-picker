use super::*;
use pwe_domain::Size;

const SLEEP_IN_A_WHOLE_WINDOW: &str = include_str!("../fixtures/sleep-in-a-whole-window.json");

#[test]
fn a_real_key_press_reads_as_its_session_block_label_program_and_folder() {
    let facts = parse_key_facts(SLEEP_IN_A_WHOLE_WINDOW).expect("facts");

    assert_eq!(
        facts,
        KeyFacts {
            session: "session:11111111-1111-7111-8111-111111111111".to_string(),
            block: "block:22222222-2222-7222-8222-222222222222".to_string(),
            label: "shell".to_string(),
            program: "sleep".to_string(),
            folder: "/Users/ada/src/project".to_string(),
            window: Size {
                columns: 80,
                rows: 24
            },
        }
    );
}

#[test]
fn facts_that_name_no_block_are_refused() {
    let without_block = SLEEP_IN_A_WHOLE_WINDOW.replacen("\"block\":", "\"other\":", 1);

    assert!(parse_key_facts(&without_block).is_err());
    assert!(parse_key_facts("not json").is_err());
}

#[test]
fn a_block_the_view_does_not_hold_has_no_label() {
    let elsewhere =
        SLEEP_IN_A_WHOLE_WINDOW.replacen("block:22222222-2222", "block:00000000-0000", 1);

    assert_eq!(parse_key_facts(&elsewhere).expect("facts").label, "");
}
