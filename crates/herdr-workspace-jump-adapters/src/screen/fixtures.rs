use herdr_workspace_jump_domain::{Frame, JumpTarget, Row};

pub(super) fn target(label: &str, key: char) -> JumpTarget {
    JumpTarget {
        action_id: format!("jump_{label}"),
        label: label.to_string(),
        directory: format!("/opt/{label}"),
        pick_key: key,
    }
}

pub(super) fn three_workspaces() -> Vec<JumpTarget> {
    vec![
        target("Ivy", 'i'),
        target("homelab", 'h'),
        target("dotfiles", 'f'),
    ]
}

pub(super) fn browsing(
    targets: &[JumpTarget],
    selected: usize,
    current: Option<usize>,
) -> Frame<'_> {
    Frame {
        rows: targets
            .iter()
            .enumerate()
            .map(|(index, target)| Row {
                target,
                selected: index == selected,
                current: Some(index) == current,
            })
            .collect(),
    }
}
