use std::path::PathBuf;

use pwe_adapters::RexPanes;

use super::*;
use crate::cli_command::{RecordedCli, spawning};

const NO_RECORDS: &str = r#"{"records":[]}"#;
const BASH_IN_FRONT: &str = r#"{"foreground":{"name":"bash","cwd":"/work/repo"}}"#;
const CLAUDE_IN_FRONT: &str = r#"{"foreground":{"name":"claude","cwd":"/work/repo"}}"#;

fn rex_over(process: &str, screen: &str) -> RecordedCli {
    RecordedCli::rex(&[
        (
            "--autostart=false block call -b block:2 program_status",
            NO_RECORDS,
        ),
        ("--autostart=false block call -b block:2 process", process),
        ("--autostart=false capture -b block:2 --trim", screen),
        ("--autostart=false send -b block:2 -- cd /work/repo/src", ""),
        ("--autostart=false send -b block:2 -- src/main.rs", ""),
        ("--autostart=false send-key -b block:2 enter", ""),
    ])
}

fn hand(fake: &RecordedCli, instruction: Option<&str>) -> Result<(), CommandError> {
    let mut panes = RexPanes::at(PathBuf::from(&fake.binary), None);
    hand_to_block(&mut panes, "block:2", instruction.map(str::to_string))
}

fn sent(fake: &RecordedCli) -> Vec<Vec<String>> {
    fake.calls()
        .into_iter()
        .filter(|call| call[1].starts_with("send"))
        .collect()
}

#[test]
fn a_folder_picked_over_a_shell_block_is_typed_as_cd_and_entered() {
    let _spawning = spawning();
    let fake = rex_over(BASH_IN_FRONT, "~/repo\n$ \n");

    assert!(hand(&fake, Some("cd\t/work/repo/src")).is_ok());
    assert_eq!(
        sent(&fake),
        [
            vec![
                "--autostart=false",
                "send",
                "-b",
                "block:2",
                "--",
                "cd /work/repo/src"
            ],
            vec!["--autostart=false", "send-key", "-b", "block:2", "enter"],
        ]
    );
}

#[test]
fn text_picked_over_an_idle_claude_is_typed_without_enter() {
    let _spawning = spawning();
    let fake = rex_over(CLAUDE_IN_FRONT, "───\n❯ \n───\n");

    assert!(hand(&fake, Some("insert\tsrc/main.rs")).is_ok());
    assert_eq!(
        sent(&fake),
        [vec![
            "--autostart=false",
            "send",
            "-b",
            "block:2",
            "--",
            "src/main.rs"
        ]]
    );
}

#[test]
fn a_draft_in_claudes_input_line_refuses_and_types_nothing() {
    let _spawning = spawning();
    let fake = rex_over(CLAUDE_IN_FRONT, "───\n❯ fix the flaky\n───\n");

    let refused = hand(&fake, Some("run\tgit status"));

    assert!(
        matches!(&refused, Err(CommandError::Failed(reason)) if reason.contains("fix the flaky")),
        "{refused:?}"
    );
    assert!(sent(&fake).is_empty());
}

#[test]
fn closing_the_picker_hands_nothing_over() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[]);

    assert!(hand(&fake, None).is_ok());
}
