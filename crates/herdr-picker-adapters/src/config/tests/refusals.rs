use super::*;

mod picker;

#[test]
fn read_config_refuses_a_missing_file() {
    let sandbox = TempDirectory::new("missing");
    let path = sandbox.0.join("config.toml");

    let refusal = read_config(&path).expect_err("a missing file is refused");

    assert!(
        matches!(refusal, ConfigError::Unreadable { .. }),
        "{refusal}"
    );
    assert!(refusal.to_string().contains(&path.display().to_string()));
}

#[test]
fn read_config_refuses_unparseable_toml_and_names_the_line() {
    let (refusal, rendered) = refusal_for(
        "unparseable",
        "[workspaces]\nnetpulse = \"/opt/netpulse\"\nhomelab = unquoted\n",
    );

    assert!(matches!(
        refusal,
        ConfigError::Unparseable { line: Some(3), .. }
    ));
    assert!(rendered.contains("at line 3"), "{rendered}");
}

#[test]
fn read_config_refuses_an_unknown_table() {
    let (refusal, rendered) = refusal_for("no-table", "[other]\nkey = \"value\"\n");

    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
    assert!(rendered.contains("unknown field"), "{rendered}");
}

#[test]
fn read_config_refuses_an_unknown_top_level_key() {
    let (_, rendered) = refusal_for(
        "unknown-key",
        "[workspace]\nnetpulse = \"/opt/netpulse\"\n\n[workspaces]\nivy = \"/opt/ivy\"\n",
    );

    assert!(rendered.contains("unknown field `workspace`"), "{rendered}");
}

#[test]
fn read_config_refuses_an_empty_workspaces_table() {
    let (refusal, rendered) = refusal_for("empty-table", "[workspaces]\n");

    assert_eq!(
        refusal_cause(refusal),
        TargetError::NoWorkspaces,
        "{rendered}"
    );
    assert!(rendered.contains("declares no workspace"), "{rendered}");
}

#[test]
fn read_config_refuses_an_empty_label_or_an_empty_directory() {
    let (empty_label, rendered) =
        refusal_for("empty-label", "[workspaces]\n\"\" = \"/opt/project\"\n");
    assert_eq!(refusal_cause(empty_label), TargetError::EmptyLabel);
    assert!(rendered.contains("label is empty"), "{rendered}");

    let (empty_directory, rendered) =
        refusal_for("empty-directory", "[workspaces]\nnetpulse = \"\"\n");
    assert_eq!(
        refusal_cause(empty_directory),
        TargetError::EmptyDirectory {
            label: "netpulse".to_string()
        }
    );
    assert!(rendered.contains("empty directory"), "{rendered}");
}

#[test]
fn read_config_refuses_two_labels_that_derive_the_same_action_id() {
    let (refusal, rendered) = refusal_for(
        "collision",
        "[workspaces]\n\"my-project\" = \"/opt/one\"\n\"my_project\" = \"/opt/two\"\n",
    );

    assert_eq!(
        refusal_cause(refusal),
        TargetError::CollidingActionId {
            first: "my-project".to_string(),
            second: "my_project".to_string(),
            action_id: "jump_my_project".to_string(),
        }
    );
    assert!(rendered.contains("jump_my_project"), "{rendered}");
}

fn refusal_cause(refusal: ConfigError) -> TargetError {
    match refusal {
        ConfigError::Invalid { cause, .. } => cause,
        other => panic!("expected a declared-workspace refusal, got {other:?}"),
    }
}

#[test]
fn read_config_refuses_a_table_without_a_directory_or_with_an_unknown_field() {
    let (refusal, rendered) = refusal_for("no-dir", "[workspaces]\nIvy = { key = \"v\" }\n");
    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
    assert!(
        rendered.contains("a directory, or a table of a dir and an optional key"),
        "the refusal says what a declaration may be: {rendered}"
    );

    let (refusal, rendered) = refusal_for(
        "unknown-field",
        "[workspaces]\nIvy = { dir = \"/opt/ivy\", keys = \"v\" }\n",
    );
    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
}

#[test]
fn read_config_refuses_two_workspaces_that_pick_on_the_same_key() {
    let (refusal, rendered) = refusal_for(
        "key-collision",
        "[workspaces]\ndamnit = \"/opt/damnit\"\ndotfiles = \"/opt/dotfiles\"\n",
    );

    assert_eq!(
        refusal_cause(refusal),
        TargetError::CollidingPickKey {
            first: "damnit".to_string(),
            second: "dotfiles".to_string(),
            key: 'd',
        }
    );
    assert!(rendered.contains("damnit"), "{rendered}");
    assert!(rendered.contains("dotfiles"), "{rendered}");
}

#[test]
fn read_config_refuses_a_pick_key_that_is_not_one_printable_character() {
    let (refusal, rendered) = refusal_for(
        "long-key",
        "[workspaces]\nIvy = { dir = \"/opt/ivy\", key = \"vv\" }\n",
    );

    assert_eq!(
        refusal_cause(refusal),
        TargetError::UnusablePickKey {
            label: "Ivy".to_string(),
            key: "vv".to_string(),
        },
        "{rendered}"
    );
}
