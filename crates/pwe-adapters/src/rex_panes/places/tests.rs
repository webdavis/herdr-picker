use std::path::PathBuf;

use pwe_application::{PanePlaces, PlacedPane};
use pwe_domain::Pane;

use crate::RexPanes;
use crate::cli_command::{RecordedCli, spawning};

const FIRST: &str = "session:00000000-0000-4000-8000-00000000000a";
const SECOND: &str = "session:00000000-0000-4000-8000-00000000000b";
const SESSIONS: &str = r#"{"sessions":[{"session_id":"session:00000000-0000-4000-8000-00000000000a"},{"session_id":"session:00000000-0000-4000-8000-00000000000b"}]}"#;
const FIRST_VIEW: &str = r#"{"active_window_id":"window:00000000-0000-4000-8000-0000000000d1","windows":[{"window_id":"window:00000000-0000-4000-8000-0000000000d1","label":"api","focused_block_id":"block:00000000-0000-4000-8000-0000000000b1","layers":[{"blocks":[{"block_id":"block:00000000-0000-4000-8000-0000000000b1"}]}]}]}"#;
const SECOND_VIEW: &str = r#"{"active_window_id":"window:00000000-0000-4000-8000-0000000000d2","windows":[{"window_id":"window:00000000-0000-4000-8000-0000000000d2","label":"docs","focused_block_id":"block:00000000-0000-4000-8000-0000000000b2","layers":[{"blocks":[{"block_id":"block:00000000-0000-4000-8000-0000000000b2"}]}]}]}"#;

fn rex_with_two_sessions() -> RecordedCli {
    let view = |session: &str| {
        format!("--autostart=false api call session.view {{\"session_id\":\"{session}\"}}")
    };
    RecordedCli::rex(&[
        ("--autostart=false ls --json", SESSIONS),
        (&view(FIRST), FIRST_VIEW),
        (&view(SECOND), SECOND_VIEW),
    ])
}

fn pane(session: &str, tab: &str, block: &str) -> Pane {
    Pane {
        rex_session: session.to_string(),
        tab: format!("window:00000000-0000-4000-8000-0000000000{tab}"),
        block: format!("block:00000000-0000-4000-8000-0000000000{block}"),
    }
}

#[test]
fn the_focused_pane_is_read_from_the_sessions_view() {
    let _spawning = spawning();
    let fake = rex_with_two_sessions();

    let focused = RexPanes::at(PathBuf::from(&fake.binary), None).focused_pane(SECOND);

    assert_eq!(focused, Ok(pane(SECOND, "d2", "b2")));
}

#[test]
fn a_block_is_placed_by_looking_through_every_session() {
    let _spawning = spawning();
    let fake = rex_with_two_sessions();

    let placed = RexPanes::at(PathBuf::from(&fake.binary), None)
        .placed_pane("block:00000000-0000-4000-8000-0000000000b2");

    assert_eq!(
        placed,
        Ok(Some(PlacedPane {
            pane: pane(SECOND, "d2", "b2"),
            tab_label: "docs".to_string(),
        }))
    );
}

#[test]
fn a_block_rex_does_not_hold_is_not_placed() {
    let _spawning = spawning();
    let fake = rex_with_two_sessions();

    let placed = RexPanes::at(PathBuf::from(&fake.binary), None).placed_pane("block:gone");

    assert_eq!(placed, Ok(None));
}
