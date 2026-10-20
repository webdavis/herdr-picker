use herdr_picker_domain::{Action, PickerCall, Preview};

use super::*;
use crate::builtin::rows::key;

const PEOPLE: &str = r#"[
  {"id": "P1:ABPerson", "name": "Ada Lovelace", "fields": [
    {"id": "firstName", "group": "name", "label": "First name", "value": "Ada"},
    {"id": "ph1", "group": "phone", "label": "mobile", "value": "+1 (555) 010-2000"},
    {"id": "em1", "group": "email", "label": "home", "value": "ada@example.com"},
    {"id": "modificationDate", "group": "other", "label": "Modified", "value": "2026-10-19T12:07:44.000Z"}
  ]},
  {"id": "P2:ABPerson", "name": "", "fields": [
    {"id": "ph1", "group": "phone", "label": "work", "value": "555-0100"},
    {"id": "ph2", "group": "phone", "label": "home", "value": "555 0101 ext"}
  ]}
]"#;

fn people() -> Vec<Person> {
    serde_json::from_str(PEOPLE).expect("people")
}

fn next(name: &str, args: &[&str]) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state: None,
        query: None,
    })
}

const CARD: &str =
    "First name: Ada\nmobile: +1 (555) 010-2000\nhome: ada@example.com\nModified: 2026-10-19 12:07";

#[test]
fn a_contact_copies_its_whole_card_and_shows_it() {
    let rows = contact_rows(&people(), "all", 0);

    assert_eq!(rows[0].label, "Ada Lovelace");
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(format!(
            "{CARD}\n\nCopy: all. Enter copies; Alt-Y copies and stays."
        )))
    );
    assert_eq!(rows[0].enter, Action::Copy(CARD.to_string()));
    assert_eq!(
        rows[0].action(&key("alt-y")),
        Some(&Action::CopyStay(CARD.to_string()))
    );
    assert_eq!(rows[1].label, "(unnamed contact)");
}

#[test]
fn alt_digits_choose_which_fields_enter_copies() {
    let rows = contact_rows(&people(), "phone", 0);

    assert_eq!(
        rows[0].action(&key("alt-3")),
        Some(&Action::State(r#""email""#.to_string()))
    );
    assert_eq!(
        rows[0].action(&key("alt-0")),
        Some(&Action::State(r#""all""#.to_string()))
    );
    assert_eq!(rows[0].enter, Action::Copy("+1 (555) 010-2000".to_string()));
    assert_eq!(
        rows[1].enter,
        Action::Copy("555-0100\n555 0101 ext".to_string())
    );
}

#[test]
fn ctrl_v_calls_a_lone_number_and_offers_a_choice_of_several() {
    let rows = contact_rows(&people(), "all", 0);

    assert_eq!(
        rows[0].action(&key("ctrl-v")),
        Some(&Action::Open("tel:+15550102000".to_string()))
    );
    assert_eq!(
        rows[1].action(&key("ctrl-v")),
        Some(&next("contact-call", &["P2:ABPerson"]))
    );
}

#[test]
fn alt_7_chooses_fields_and_alt_m_finds_the_contacts_conversations() {
    let rows = contact_rows(&people(), "all", 0);

    assert_eq!(
        rows[0].action(&key("alt-7")),
        Some(&next("contact-fields", &["P1:ABPerson"]))
    );
    assert_eq!(
        rows[0].action(&key("alt-m")),
        Some(&next("chats", &["+1 (555) 010-2000", "ada@example.com"]))
    );
}

#[test]
fn a_contacts_fields_are_rows_that_copy_their_value() {
    let rows = field_rows(&people(), &["P1:ABPerson"], 0);

    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows[1].label,
        "\u{1b}[2mmobile:\u{1b}[0m \u{1b}[38;2;166;227;161m+1 (555) 010-2000\u{1b}[0m"
    );
    assert_eq!(
        rows[1].preview,
        Some(Preview::Text("mobile of Ada Lovelace".to_string()))
    );
    assert_eq!(rows[1].enter, Action::Copy("+1 (555) 010-2000".to_string()));
    assert_eq!(rows[1].group, Some("phone".to_string()));
    assert_eq!(rows[0].label, "\u{1b}[2mFirst name:\u{1b}[0m Ada");
}

#[test]
fn calling_offers_each_phone_number_and_refuses_one_that_is_no_telephone_url() {
    let rows = call_rows(&people(), &["P2:ABPerson"], 0);

    assert_eq!(rows[0].enter, Action::Open("tel:5550100".to_string()));
    assert_eq!(rows[1].enter, Action::Nothing);
    assert_eq!(
        rows[1].preview,
        Some(Preview::Text(
            "The selected phone number cannot be used as a telephone URL.".to_string()
        ))
    );
}
