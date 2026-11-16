use super::*;

const SESSION: &str = "session:00000000-0000-4000-8000-0000000000a1";
const TWO_TABS: &str = r#"{"session_id":"session:00000000-0000-4000-8000-0000000000a1","active_window_id":"window:two","windows":[
    {"window_id":"window:one","label":"api","focused_block_id":"block:one","layers":[{"layer_id":"layer:one","blocks":[{"block_id":"block:one"},{"block_id":"block:three"}]}]},
    {"window_id":"window:two","label":"docs","focused_block_id":"block:two","layers":[{"layer_id":"layer:two","blocks":[{"block_id":"block:two"}]}]}
]}"#;

fn pane(tab: &str, block: &str) -> Pane {
    Pane {
        rex_session: SESSION.to_string(),
        tab: tab.to_string(),
        block: block.to_string(),
    }
}

#[test]
fn the_focused_pane_is_the_active_tabs_focused_block() {
    assert_eq!(
        focused_pane_in_view(SESSION, TWO_TABS),
        Ok(pane("window:two", "block:two"))
    );
}

#[test]
fn a_view_without_an_active_tab_names_no_focused_pane() {
    assert!(focused_pane_in_view(SESSION, r#"{"windows":[]}"#).is_err());
}

#[test]
fn a_block_is_placed_in_its_tab_with_the_tabs_label() {
    assert_eq!(
        placed_pane_in_view(SESSION, TWO_TABS, "block:three"),
        Ok(Some((pane("window:one", "block:three"), "api".to_string())))
    );
}

#[test]
fn a_block_in_no_tab_of_the_session_is_not_placed() {
    assert_eq!(
        placed_pane_in_view(SESSION, TWO_TABS, "block:elsewhere"),
        Ok(None)
    );
}

#[test]
fn a_tab_rex_named_from_its_id_is_named_by_its_position() {
    let view = r#"{"windows":[
        {"window_id":"window:00000000-0000-4000-8000-0000000000d1","label":"api","layers":[]},
        {"window_id":"window:00000000-0000-4000-8000-0000000000d2","label":"Window 000000d2","layers":[{"blocks":[{"block_id":"block:two"}]}]}
    ]}"#;

    let placed = placed_pane_in_view(SESSION, view, "block:two");

    assert_eq!(
        placed.map(|placed| placed.map(|(_, tab_label)| tab_label)),
        Ok(Some("2".to_string()))
    );
}
