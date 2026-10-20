use herdr_picker_domain::{Action, PickerRow, Preview};
use serde_json::json;

use super::*;
use crate::builtin::rows::key;

fn listed() -> Vec<Value> {
    vec![
        json!({"id": "W1:T1", "window": "W1", "tab": "T1", "title": "Linear", "url": "https://www.linear.app/team",
               "space": {"id": "S1", "title": "Work"}, "location": "pinned"}),
        json!({"id": "W1:T2", "window": "W1", "tab": "T2", "title": "", "url": "about:blank",
               "space": null, "location": ""}),
    ]
}

#[test]
fn an_arc_tab_shows_its_title_and_site_and_enter_switches_to_it() {
    let rows = tab_rows(&listed(), "arc", "herdr-picker");

    assert_eq!(
        rows[0],
        PickerRow {
            label: "Linear  \u{1b}[2mlinear.app\u{1b}[0m".to_string(),
            preview: Some(Preview::Text(
                "Linear\n\nURL      https://www.linear.app/team\nSpace    Work\nSection  Pinned\nBrowser  Arc\n\nEnter switches to this tab, Alt-Y copies the URL, Alt-B switches to Chrome.".to_string()
            )),
            enter: Action::Run("herdr-picker tabs focus arc W1 T1 S1".to_string()),
            keys: vec![
                (key("alt-y"), Action::CopyStay("https://www.linear.app/team".to_string())),
                (key("alt-b"), Action::State(r#""chrome""#.to_string())),
            ],
            group: Some("Work".to_string()),
        }
    );
}

#[test]
fn a_tab_outside_a_space_is_named_by_its_url_and_switched_to_without_one() {
    let rows = tab_rows(&listed(), "chrome", "herdr-picker");

    assert_eq!(rows[1].label, "about:blank");
    assert_eq!(
        rows[1].enter,
        Action::Run("herdr-picker tabs focus chrome W1 T2".to_string())
    );
    assert_eq!(
        rows[1].keys[1],
        (key("alt-b"), Action::State(r#""arc""#.to_string()))
    );
    assert_eq!(rows[1].group, None);
    assert!(
        matches!(&rows[1].preview, Some(Preview::Text(text)) if text.starts_with("about:blank\n\nURL      about:blank\nBrowser  Chrome\n\n"))
    );
}

#[test]
fn a_browser_that_cannot_answer_is_one_row_saying_why() {
    assert_eq!(
        unanswered("arc is not running", "arc").label,
        "\u{1b}[2mArc is not running\u{1b}[0m"
    );
    assert_eq!(
        unanswered("Application can't be found.", "arc").label,
        "\u{1b}[2mArc is not installed\u{1b}[0m"
    );
    assert_eq!(
        unanswered("chrome is not running", "chrome").keys,
        vec![(key("alt-b"), Action::State(r#""arc""#.to_string()))]
    );
    assert_eq!(
        unanswered("Not authorized to send Apple events to Arc. (-1743)", "arc").label,
        "\u{1b}[2mArc did not answer\u{1b}[0m"
    );
}
