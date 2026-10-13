use super::*;
use herdr_workspace_jump_application::JumpError;

struct Herdr(Result<Vec<Workspace>, JumpError>);

impl WorkspaceDirectory for Herdr {
    fn list(&mut self) -> Result<Vec<Workspace>, JumpError> {
        self.0.clone()
    }

    fn focus(&mut self, _: &str) -> Result<(), JumpError> {
        Err(JumpError::Unreachable(
            "the popup never focuses".to_string(),
        ))
    }

    fn create(&mut self, _: &str, _: &str) -> Result<(), JumpError> {
        Err(JumpError::Unreachable(
            "the popup never creates".to_string(),
        ))
    }
}

fn layout() -> Layout {
    Layout::new(ReadlineKeymap::LEGEND, &[target()])
}

#[test]
fn what_herdr_lists_is_what_the_menu_marks_open_and_the_layout_says_nothing() {
    let listed = vec![Workspace {
        workspace_id: "w11".to_string(),
        label: "Ivy".to_string(),
    }];
    let mut herdr = Herdr(Ok(listed.clone()));

    let (live, layout) = open_workspaces(&mut herdr, layout());

    assert_eq!(live, listed);
    assert_eq!(layout.notice(), None);
}

#[test]
fn when_herdr_does_not_answer_nothing_is_marked_open_and_the_layout_says_so() {
    let mut herdr = Herdr(Err(JumpError::Unreachable("no socket".to_string())));

    let (live, layout) = open_workspaces(&mut herdr, layout());

    assert!(live.is_empty());
    assert_eq!(layout.notice(), Some(HERDR_DID_NOT_ANSWER));
}

fn target() -> JumpTarget {
    JumpTarget {
        action_id: "jump_ivy".to_string(),
        label: "Ivy".to_string(),
        directory: "~/workspaces/Ivy".to_string(),
        pick_key: 'i',
    }
}

#[test]
fn the_spawned_jump_is_this_binary_with_the_label_the_directory_and_this_pid() {
    let mut spawned = Vec::new();
    let this_binary = env::current_exe().expect("a running binary has a path");

    let started = spawn_jump(&target(), |binary, arguments| {
        assert_eq!(binary, this_binary, "the popup spawned something else");
        spawned = arguments.iter().map(|word| word.to_string()).collect();
        Ok(())
    });

    assert!(started.is_ok(), "{started:?}");
    assert_eq!(
        spawned,
        vec![
            "jump",
            "Ivy",
            "~/workspaces/Ivy",
            "--after-pid",
            &process::id().to_string()
        ]
    );
}

#[test]
fn a_spawn_that_fails_is_reported_rather_than_swallowed() {
    let refused = spawn_jump(&target(), |_, _| Err(io::Error::other("no such binary")));

    assert!(
        matches!(&refused, Err(CommandError::Failed(reason)) if reason.contains("no such binary")),
        "{refused:?}"
    );
}
