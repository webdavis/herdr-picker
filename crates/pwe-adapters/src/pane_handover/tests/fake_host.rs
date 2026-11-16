use std::path::Path;

use pwe_application::JumpError;
use pwe_domain::{HerdrCall, NewPane};
use pwe_protocol::{PaneInfo, Refusal};

use crate::pane_handover::{HandoverError, PaneHost, hand_over};

struct FakeHost {
    pane: PaneInfo,
    screen: String,
    done: Vec<String>,
}

impl FakeHost {
    fn over(agent: Option<&str>, status: &str, screen: &str) -> Self {
        Self {
            pane: PaneInfo {
                pane_id: "p2".to_string(),
                workspace_id: String::new(),
                tab_id: String::new(),
                cwd: "/nowhere/repo".to_string(),
                agent: agent.map(str::to_string),
                status: status.to_string(),
            },
            screen: screen.to_string(),
            done: Vec::new(),
        }
    }
}

impl PaneHost for FakeHost {
    fn pane(&mut self, target: &str) -> Result<PaneInfo, JumpError> {
        self.done.push(format!("read {target}"));
        Ok(self.pane.clone())
    }

    fn screen(&mut self, target: &str) -> Result<String, JumpError> {
        self.done.push(format!("screen {target}"));
        Ok(self.screen.clone())
    }

    fn type_text(&mut self, target: &str, text: &str) -> Result<(), JumpError> {
        self.done.push(format!("type {target} {text}"));
        Ok(())
    }

    fn press_enter(&mut self, target: &str) -> Result<(), JumpError> {
        self.done.push(format!("enter {target}"));
        Ok(())
    }

    fn open_pane(
        &mut self,
        beside: &PaneInfo,
        place: NewPane,
        folder: &str,
    ) -> Result<String, JumpError> {
        self.done
            .push(format!("open {place:?} of {} in {folder}", beside.pane_id));
        Ok("p9".to_string())
    }

    fn host_call(&mut self, call: &HerdrCall, _queue: &Path) -> Result<(), HandoverError> {
        self.done.push(format!("host {call:?}"));
        Ok(())
    }
}

fn hand(host: &mut FakeHost, line: &str) -> Result<(), HandoverError> {
    hand_over(host, "p2", line, "nvim", Path::new("/nowhere/queue"))
}

#[test]
fn a_cd_into_a_shell_is_typed_and_entered() {
    let mut host = FakeHost::over(None, "unknown", "~/repo\n❯ \n");

    assert_eq!(hand(&mut host, "cd\tsrc"), Ok(()));
    assert_eq!(
        host.done,
        ["read p2", "screen p2", "type p2 cd src", "enter p2"]
    );
}

#[test]
fn an_insert_into_a_shell_is_typed_after_any_draft_without_reading_the_screen() {
    let mut host = FakeHost::over(None, "unknown", "~/repo\n❯ git sta\n");

    assert_eq!(hand(&mut host, "insert\tsrc/main.rs"), Ok(()));
    assert_eq!(host.done, ["read p2", "type p2 src/main.rs"]);
}

#[test]
fn a_blocked_claude_refuses_and_nothing_is_typed() {
    let mut host = FakeHost::over(Some("claude"), "blocked", "❯ \n");

    assert_eq!(
        hand(&mut host, "run\tgit status"),
        Err(HandoverError::Refused(Refusal::Blocked))
    );
    assert_eq!(host.done, ["read p2", "screen p2"]);
}

#[test]
fn a_command_for_a_new_pane_opens_it_in_the_folder_and_runs_there() {
    let mut host = FakeHost::over(None, "unknown", "");

    assert_eq!(hand(&mut host, "right\trun\tnvim a.rs"), Ok(()));
    assert_eq!(
        host.done,
        [
            "read p2",
            "open Right of p2 in /nowhere/repo",
            "type p9 nvim a.rs",
            "enter p9"
        ]
    );
}

#[test]
fn a_host_only_call_goes_to_the_host() {
    let mut host = FakeHost::over(None, "unknown", "");

    assert_eq!(hand(&mut host, "herdr\t{\"focus_pane\":\"p7\"}"), Ok(()));
    assert_eq!(host.done, ["read p2", "host FocusPane(\"p7\")"]);
}
