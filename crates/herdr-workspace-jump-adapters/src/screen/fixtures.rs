use herdr_workspace_jump_domain::{Frame, JumpTarget, Legend, Row, SearchLine};

pub(super) const LEGEND: Legend = Legend {
    search: "/ find",
    keys: "the keys",
};

pub(super) fn three_workspaces() -> Vec<JumpTarget> {
    vec![
        target("Ivy", 'i'),
        target("homelab", 'h'),
        target("dotfiles", 'f'),
    ]
}

pub(super) fn browsing(targets: &[JumpTarget], selected: usize) -> Frame<'_> {
    Frame {
        search: None,
        rows: targets
            .iter()
            .enumerate()
            .map(|(index, target)| Row {
                target,
                selected: index == selected,
            })
            .collect(),
        total: targets.len(),
    }
}

pub(super) fn searching<'a>(text: &str, cursor: usize, shown: &'a [JumpTarget]) -> Frame<'a> {
    let mut frame = browsing(shown, 0);
    frame.search = Some(SearchLine {
        text: text.to_string(),
        cursor,
    });
    frame.total = three_workspaces().len();
    frame
}

fn target(label: &str, key: char) -> JumpTarget {
    JumpTarget {
        action_id: format!("jump_{label}"),
        label: label.to_string(),
        directory: format!("/opt/{label}"),
        pick_key: key,
    }
}
