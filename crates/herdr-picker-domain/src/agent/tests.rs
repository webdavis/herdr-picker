use super::*;

fn agent(pane_id: &str, tab_id: &str, status: &str) -> Agent {
    Agent {
        pane_id: pane_id.to_string(),
        workspace_id: "w1".to_string(),
        tab_id: tab_id.to_string(),
        kind: "claude".to_string(),
        title: pane_id.to_string(),
        cwd: "/repo".to_string(),
        status: AgentStatus::parse(status),
        since: None,
    }
}

fn panes(agents: &[Agent]) -> Vec<&str> {
    agents.iter().map(|agent| agent.pane_id.as_str()).collect()
}

#[test]
fn needs_you_then_done_then_the_most_recently_visited() {
    let agents = [
        agent("p1", "t1", "idle"),
        agent("p2", "t2", "working"),
        agent("p3", "t3", "done"),
        agent("p4", "t4", "blocked"),
    ];

    assert_eq!(
        panes(&ordered(&agents, &["p2", "p1"], None)),
        vec!["p4", "p3", "p2", "p1"]
    );
}

#[test]
fn the_current_tab_never_comes_first_even_when_it_needs_you() {
    let agents = [agent("p1", "t1", "blocked"), agent("p2", "t2", "idle")];

    assert_eq!(panes(&ordered(&agents, &[], Some("t1"))), vec!["p2", "p1"]);
}

#[test]
fn unvisited_agents_keep_herdrs_order() {
    let agents = [agent("p1", "t1", "idle"), agent("p2", "t2", "unknown")];

    assert_eq!(panes(&ordered(&agents, &[], None)), vec!["p1", "p2"]);
}

#[test]
fn statuses_read_and_draw_as_herdr_does() {
    let drawn: Vec<(&str, &str)> = ["blocked", "working", "done", "idle", "starting"]
        .map(|word| {
            let status = AgentStatus::parse(word);
            (status.word(), status.glyph())
        })
        .to_vec();

    assert_eq!(
        drawn,
        vec![
            ("blocked", "◉"),
            ("working", "◐"),
            ("done", "●"),
            ("idle", "✓"),
            ("unknown", "·"),
        ]
    );
}

#[test]
fn only_a_finished_turn_takes_a_queued_prompt() {
    let settled: Vec<bool> = ["blocked", "working", "done", "idle", "unknown"]
        .map(|word| AgentStatus::parse(word).settled())
        .to_vec();

    assert_eq!(settled, vec![false, false, true, true, false]);
}

#[test]
fn an_age_reads_in_its_largest_whole_unit() {
    assert_eq!(
        [45, 180, 7200, 345_600].map(age),
        ["45s", "3m", "2h", "4d"].map(str::to_string)
    );
}

#[test]
fn the_changed_agent_starts_its_clock_and_unchanged_ones_keep_theirs() {
    let before = [
        Agent {
            since: Some(100),
            ..agent("p1", "t1", "working")
        },
        Agent {
            since: Some(200),
            ..agent("p2", "t2", "working")
        },
    ];
    let now = [
        agent("p1", "t1", "working"),
        agent("p2", "t2", "idle"),
        agent("p3", "t3", "idle"),
    ];

    let since: Vec<Option<u64>> = carry_since(now.to_vec(), &before, "p3", 500)
        .iter()
        .map(|agent| agent.since)
        .collect();

    assert_eq!(since, vec![Some(100), None, Some(500)]);
}
