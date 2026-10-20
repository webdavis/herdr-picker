use herdr_picker_domain::Action;

use crate::builtin::Asked;
use crate::builtin::data::tests::{ACCOUNT, holding, on};
use crate::builtin::git::fixture::read;
use crate::builtin::tests::asked;

#[test]
fn the_format_menu_copies_with_secrets_hidden_and_alt_u_shows_them() {
    let tree = holding("data-copy", "a.json", ACCOUNT);
    let path = tree.path("a.json");
    let asked_for = Asked {
        state: Some("[]"),
        ..Asked::default()
    };

    let (header, rows) =
        read(&asked("data-copy", &[&path], &asked_for, &tree.places("")).expect("rows"));

    assert_eq!(
        header,
        Some("Copy . as: secrets hidden, Alt-U copies them visible.".to_string())
    );
    let hidden = concat!(
        "{\n  \"workspace\": {\n    \"ignoreGlobs\": [\n      \"*.log\",\n      \"dist\"\n    ]\n  },\n",
        "  \"password\": \"[hidden]\",\n  \"payload\": \"eyJ1c2VyIjoiYW5uIn0=\"\n}"
    );
    assert_eq!(rows[0].label, "JSON");
    assert_eq!(rows[0].enter, Action::Copy(hidden.to_string()));
    assert_eq!(
        on(&rows[0], "alt-u"),
        Action::Copy(hidden.replace("[hidden]", "hunter2"))
    );
    assert_eq!(rows[1].label, "YAML");
    assert_eq!(
        rows[1].enter,
        Action::Copy(
            "workspace:\n  ignoreGlobs:\n    - '*.log'\n    - dist\npassword: '[hidden]'\npayload: eyJ1c2VyIjoiYW5uIn0="
                .to_string()
        )
    );
    assert!(rows[2].label.contains("TOON"), "{}", rows[2].label);
}
