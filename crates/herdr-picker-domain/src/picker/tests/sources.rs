use super::*;

fn scripted(sources: &[&str]) -> PickerDeclaration {
    PickerDeclaration {
        sources: sources.iter().map(|source| source.to_string()).collect(),
        ..declared("my-notes", "ctrl-t m")
    }
}

#[test]
fn a_scripted_picker_offers_the_sources_it_declares_in_order() {
    assert_eq!(
        only(scripted(&["folders", "tags", "recent"])).sources,
        ["folders", "tags", "recent"]
    );
}

#[test]
fn a_picker_is_titled_by_its_name_unless_it_declares_a_title() {
    assert_eq!(only(scripted(&[])).title, "My Notes");
    assert_eq!(
        only(PickerDeclaration {
            title: Some("Vault".to_string()),
            ..scripted(&[])
        })
        .title,
        "Vault"
    );
}

#[test]
fn a_picker_keeps_the_steps_it_declares_by_name() {
    let steps = BTreeMap::from([(
        "note-actions".to_string(),
        vec![
            "/config/pickers/notes.py".to_string(),
            "actions".to_string(),
        ],
    )]);

    assert_eq!(
        only(PickerDeclaration {
            steps: steps.clone(),
            ..scripted(&[])
        })
        .steps,
        steps
    );
}

#[test]
fn a_source_that_is_empty_or_not_lowercase_is_refused_naming_it() {
    for source in ["Tags", "", "two\tparts"] {
        assert_eq!(
            pickers([scripted(&["folders", source])])
                .expect_err("an unusable source is refused")
                .to_string(),
            format!(
                "the picker my-notes offers the source {source:?}, which is not lowercase text"
            )
        );
    }
}

#[test]
fn a_builtin_picker_declaring_sources_is_refused_since_it_reads_its_own() {
    let refusal = pickers([PickerDeclaration {
        command: PickerCommand::Builtin("files".to_string()),
        ..scripted(&["mine"])
    }])
    .expect_err("sources on a builtin are refused");

    assert_eq!(
        refusal.to_string(),
        "the picker my-notes runs a built-in picker, which reads its own sources"
    );
}
