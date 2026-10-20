use herdr_picker_adapters::{AgentSnapshot, record_place, write_snapshot};
use herdr_picker_domain::{Action, Agent, AgentStatus, HerdrCall, Key, Place, Preview, Workspace};
use herdr_picker_protocol::WireTab;
use serde_json::json;

use crate::builtin::Places;
use crate::builtin::git::fixture::{labels, read};
use crate::builtin::tests::{Tree, printed};
use crate::socket_server::{Reply, SocketServer};

mod handoff;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const PEACH: &str = "\x1b[38;2;250;179;135m";
const BLUE: &str = "\x1b[38;2;137;180;250m";
const GREEN: &str = "\x1b[38;2;166;227;161m";
const YELLOW: &str = "\x1b[38;2;249;226;175m";

// What `herdr agent list`, `workspace list` and `tab list` answered, trimmed.
pub(super) fn herdr_answers() -> Vec<Reply> {
    let agent = |pane: &str, tab: &str, status: &str, title: &str| {
        json!({"agent": "claude", "agent_status": status, "cwd": "/repo", "pane_id": pane,
               "tab_id": tab, "workspace_id": "w1", "terminal_title_stripped": title})
    };
    vec![
        Reply::Line(
            json!({"id": "req_1", "result": {"type": "agent_list", "agents": [
                agent("w1:p1", "w1:t1", "idle", "tidy"),
                agent("w1:p2", "w1:t2", "working", "build"),
                agent("w1:p3", "w1:t3", "done", "report"),
                agent("w1:p4", "w1:t4", "blocked", "review"),
            ]}})
            .to_string(),
        ),
        Reply::Line(
            json!({"id": "req_2", "result": {"workspaces": [
                {"workspace_id": "w1", "label": "dotfiles"}]}})
            .to_string(),
        ),
        Reply::Line(
            json!({"id": "req_3", "result": {"tabs": [
                {"tab_id": "w1:t1", "workspace_id": "w1", "label": "one"},
                {"tab_id": "w1:t2", "workspace_id": "w1", "label": "two"},
                {"tab_id": "w1:t3", "workspace_id": "w1", "label": "three"},
                {"tab_id": "w1:t4", "workspace_id": "w1", "label": "four"},
                {"tab_id": "w1:t5", "workspace_id": "w1", "label": "shell"}]}})
            .to_string(),
        ),
    ]
}

fn inside_herdr(tree: &Tree, server: &SocketServer) -> Places {
    let mut places = tree.places("home");
    places.herdr.socket = Some(server.path.clone());
    places
}

fn agents(source: &str, places: &Places) -> Vec<herdr_picker_domain::PickerRow> {
    read(&printed("agents", &[], Some(source), places).expect("rows")).1
}

fn row(glyph: &str, title: &str, tab: &str) -> String {
    format!("{glyph} {title}  {DIM}claude · dotfiles › {tab} · /repo{RESET}")
}

#[test]
fn everything_lists_herdrs_agents_needing_you_first_then_the_tabs_without_one() {
    let tree = Tree::new("agents-everything", &[]);
    let server = SocketServer::start(herdr_answers());

    assert_eq!(
        labels(&agents("everything", &inside_herdr(&tree, &server))),
        vec![
            row(&format!("{PEACH}◉{RESET}"), "review", "four"),
            row(&format!("{BLUE}●{RESET}"), "report", "three"),
            row(&format!("{GREEN}✓{RESET}"), "tidy", "one"),
            row(&format!("{YELLOW}◐{RESET}"), "build", "two"),
            format!("shell  {DIM}dotfiles{RESET}"),
        ]
    );
}

#[test]
fn an_agent_row_focuses_its_pane_previews_its_reply_and_closes_its_tab_after_asking() {
    let tree = Tree::new("agents-row", &[]);
    let server = SocketServer::start(herdr_answers());

    let review = agents("needs you", &inside_herdr(&tree, &server)).remove(0);

    assert_eq!(
        review.enter,
        Action::Herdr(HerdrCall::FocusPane("w1:p4".to_string()))
    );
    assert_eq!(
        review.preview,
        Some(Preview::Command(
            [
                "herdr",
                "agent",
                "read",
                "w1:p4",
                "--source",
                "recent-unwrapped",
                "--lines",
                "80"
            ]
            .map(str::to_string)
            .to_vec()
        ))
    );
    assert_eq!(
        review.keys,
        vec![
            (
                Key::parse("alt-x").expect("a key"),
                Action::Confirm {
                    question: "Close the tab four and everything running in it?".to_string(),
                    action: Box::new(Action::Herdr(HerdrCall::CloseTab("w1:t4".to_string()))),
                }
            ),
            (
                Key::parse("ctrl-r").expect("a key"),
                Action::State(r#"{"fresh":true}"#.to_string())
            ),
        ]
    );
}

fn saved(tree: &Tree) -> Places {
    let places = tree.places("home");
    let agent = |pane: &str, tab: &str, status| Agent {
        pane_id: pane.to_string(),
        workspace_id: "w1".to_string(),
        tab_id: tab.to_string(),
        kind: "codex".to_string(),
        title: pane.to_string(),
        cwd: "/repo".to_string(),
        status,
        since: None,
    };
    let tab = |id: &str, label: &str| WireTab {
        tab_id: id.to_string(),
        workspace_id: "w1".to_string(),
        label: label.to_string(),
    };
    let snapshot = AgentSnapshot {
        agents: vec![
            agent("w1:p1", "w1:t1", AgentStatus::Done),
            agent("w1:p2", "w1:t2", AgentStatus::Idle),
        ],
        workspaces: vec![Workspace {
            workspace_id: "w1".to_string(),
            label: "dotfiles".to_string(),
        }],
        tabs: vec![
            tab("w1:t1", "one"),
            tab("w1:t2", "two"),
            tab("w1:t3", "shell"),
        ],
    };
    write_snapshot(&places.herdr.snapshot, &snapshot).expect("saved");
    places
}

#[test]
fn a_fresh_saved_list_answers_without_asking_herdr_and_each_source_filters_it() {
    let tree = Tree::new("agents-saved", &[]);
    let mut places = saved(&tree);
    places.herdr.socket = Some(tree.0.join("no-herdr.sock"));
    let titles = |source| -> Vec<String> {
        agents(source, &places)
            .iter()
            .map(|row| herdr_picker_domain::unstyled(&row.label))
            .collect()
    };

    assert_eq!(
        titles("done"),
        vec!["● w1:p1  codex · dotfiles › one · /repo"]
    );
    assert_eq!(titles("terminals"), vec!["shell  dotfiles"]);
    assert_eq!(titles("workspaces"), vec!["dotfiles"]);
    assert_eq!(titles("needs you"), vec!["Nothing in needs you"]);
    assert_eq!(
        agents("workspaces", &places)[0].preview,
        Some(Preview::Text(
            "one\n  ● w1:p1\ntwo\n  ✓ w1:p2\nshell".to_string()
        ))
    );
}

#[test]
fn recent_lists_the_places_still_open_newest_first_and_enter_goes_back_there() {
    let tree = Tree::new("agents-recent", &[]);
    let mut places = saved(&tree);
    places.herdr.socket = Some(tree.0.join("no-herdr.sock"));
    let place = |at, tab: &str, pane: &str, cwd: &str| Place {
        at,
        workspace_id: "w1".to_string(),
        tab_id: tab.to_string(),
        pane_id: pane.to_string(),
        cwd: cwd.to_string(),
    };
    record_place(
        &places.herdr.places,
        place(1, "w1:t1", "w1:p1", "/repo"),
        10,
    );
    record_place(
        &places.herdr.places,
        place(2, "w1:t9", "w1:p9", "/gone"),
        10,
    );
    record_place(&places.herdr.places, place(3, "w1:t3", "w1:p5", "/tmp"), 10);

    let recent = agents("recent", &places);

    assert_eq!(
        recent
            .iter()
            .map(|row| herdr_picker_domain::unstyled(&row.label))
            .collect::<Vec<_>>(),
        vec!["  /tmp  dotfiles › shell", "● /repo  dotfiles › one"]
    );
    assert_eq!(
        recent[0].enter,
        Action::Herdr(HerdrCall::FocusPane("w1:p5".to_string()))
    );
}
