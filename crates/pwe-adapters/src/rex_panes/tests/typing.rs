use std::path::{Path, PathBuf};

use pwe_domain::{HerdrCall, NewPane};
use pwe_protocol::{PaneInfo, Refusal};

use crate::cli_command::{RecordedCli, spawning};
use crate::pane_handover::{HandoverError, PaneHost};
use crate::rex_panes::RexPanes;

const SPLIT: &str = r#"{"block_ids":["block:9"]}"#;

fn beside() -> PaneInfo {
    PaneInfo {
        pane_id: "block:2".to_string(),
        workspace_id: String::new(),
        tab_id: String::new(),
        cwd: "/work/repo".to_string(),
        agent: None,
        status: "unknown".to_string(),
    }
}

fn panes(fake: &RecordedCli) -> RexPanes {
    RexPanes::at(PathBuf::from(&fake.binary), Some("session:1".to_string()))
}

#[test]
fn typing_runs_rex_send_on_the_target_block() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[("--autostart=false send -b block:2 -- -n cd src", "")]);

    assert_eq!(panes(&fake).type_text("block:2", "-n cd src"), Ok(()));
    assert_eq!(
        fake.calls(),
        [[
            "--autostart=false",
            "send",
            "-b",
            "block:2",
            "--",
            "-n cd src"
        ]]
    );
}

#[test]
fn enter_is_one_rex_send_key_on_the_target_block() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[("--autostart=false send-key -b block:2 enter", "")]);

    assert_eq!(panes(&fake).press_enter("block:2"), Ok(()));
    assert_eq!(fake.calls().len(), 1);
}

#[test]
fn a_new_pane_to_the_left_is_a_rex_split_in_the_folder_and_answers_its_block() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[(
        "--autostart=false split -b block:2 --split left -c /srv --json",
        SPLIT,
    )]);

    assert_eq!(
        panes(&fake).open_pane(&beside(), NewPane::Left, "/srv"),
        Ok("block:9".to_string())
    );
}

#[test]
fn a_new_tab_is_a_new_rex_window_in_the_session() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[(
        "--autostart=false window new -s session:1 -c /srv --json",
        r#"{"window_id":"window:3","block_ids":["block:7"]}"#,
    )]);

    assert_eq!(
        panes(&fake).open_pane(&beside(), NewPane::Tab, "/srv"),
        Ok("block:7".to_string())
    );
}

#[test]
fn an_answer_naming_no_block_is_unreadable() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[(
        "--autostart=false split -b block:2 --split below -c /srv --json",
        "{}",
    )]);

    assert!(
        panes(&fake)
            .open_pane(&beside(), NewPane::Below, "/srv")
            .is_err()
    );
}

#[test]
fn a_call_only_herdr_can_carry_out_is_refused_in_rex() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[]);

    assert_eq!(
        panes(&fake).host_call(&HerdrCall::FocusPane("w1:p7".to_string()), Path::new("/q")),
        Err(HandoverError::Refused(Refusal::OnlyInHerdr))
    );
}
