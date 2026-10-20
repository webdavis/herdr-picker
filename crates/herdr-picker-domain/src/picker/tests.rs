use super::*;
use crate::picker_builtin::{follows_query, is_step_builtin};

fn declared(name: &str, key: &str) -> PickerDeclaration {
    PickerDeclaration {
        name: name.to_string(),
        key: key.to_string(),
        description: format!("{name} picker"),
        command: PickerCommand::Program(vec![format!("/config/pickers/{name}.sh")]),
        args: Vec::new(),
    }
}

#[test]
fn pickers_parse_each_key_into_a_chord_and_keep_the_rest() {
    let accepted = pickers([
        declared("branches", "ctrl-g b"),
        declared("files", "ctrl-t f"),
    ]);

    assert_eq!(
        accepted,
        Ok(vec![
            Picker {
                name: "branches".to_string(),
                chord: Chord::parse("ctrl-g b").expect("a chord"),
                description: "branches picker".to_string(),
                command: PickerCommand::Program(vec!["/config/pickers/branches.sh".to_string()]),
                args: Vec::new(),
            },
            Picker {
                name: "files".to_string(),
                chord: Chord::parse("ctrl-t f").expect("a chord"),
                description: "files picker".to_string(),
                command: PickerCommand::Program(vec!["/config/pickers/files.sh".to_string()]),
                args: Vec::new(),
            },
        ])
    );
}

#[test]
fn two_pickers_on_the_same_chord_are_refused_naming_both() {
    let refusal = pickers([
        declared("branches", "ctrl-t b"),
        declared("buffers", "ctrl-T  b"),
    ])
    .expect_err("a shared chord is refused");

    assert_eq!(
        refusal.to_string(),
        "the pickers branches and buffers both run on ctrl-T  b"
    );
}

#[test]
fn a_key_that_is_not_a_chord_is_refused_naming_the_picker() {
    let refusal = pickers([declared("files", "b")]).expect_err("a plain key is refused");

    assert_eq!(
        refusal,
        PickerError::UnusableKey {
            name: "files".to_string(),
            key: "b".to_string()
        }
    );
    assert_eq!(
        refusal.to_string(),
        "the picker files declares the key \"b\", which is not a chord: ctrl-<letter> or alt-<letter>, then any plain letters or punctuation, like \"ctrl-t b\""
    );
}

#[test]
fn a_name_that_a_shell_key_binding_cannot_carry_bare_is_refused() {
    let refusal =
        pickers([declared("my files", "ctrl-t f")]).expect_err("a spaced name is refused");

    assert_eq!(
        refusal.to_string(),
        "the picker name \"my files\" is not only letters, digits, - and _"
    );
    assert!(pickers([declared("", "ctrl-t f")]).is_err());
}

#[test]
fn the_builtin_pickers_are_the_ctrl_t_and_ctrl_g_ones() {
    let shipped: Vec<(&str, &str)> = BUILTIN_PICKERS
        .iter()
        .map(|builtin| (builtin.name, builtin.key))
        .collect();

    assert_eq!(
        shipped,
        vec![
            ("files", "ctrl-t f"),
            ("folders", "ctrl-t d"),
            ("bookmark", "ctrl-t ctrl-b"),
            ("status", "ctrl-g s"),
            ("history", "ctrl-g l"),
            ("branches", "ctrl-g b"),
            ("contents", "ctrl-g /"),
            ("git-config", "ctrl-g v"),
            ("recipes", "ctrl-t j"),
            ("tests", "ctrl-t t"),
            ("processes", "ctrl-t p"),
            ("network", "ctrl-t n")
        ]
    );
    assert!(
        BUILTIN_PICKERS
            .iter()
            .all(|builtin| Chord::parse(builtin.key).is_some())
    );
}

#[test]
fn a_picker_running_a_builtin_offers_its_sources_and_a_command_offers_none() {
    let running = |command: PickerCommand| Picker {
        name: "any".to_string(),
        chord: Chord::parse("ctrl-t x").expect("a chord"),
        description: String::new(),
        command,
        args: Vec::new(),
    };
    let builtin = |name: &str| running(PickerCommand::Builtin(name.to_string()));

    assert_eq!(
        builtin("files").sources(),
        ["this folder", "hidden", "home", "whole disk", "recent"]
    );
    assert_eq!(
        builtin("folders").sources(),
        ["subfolders", "parents", "git", "bookmarks"]
    );
    assert!(builtin("bookmark").sources().is_empty());
    assert!(
        running(PickerCommand::Program(vec!["files.sh".to_string()]))
            .sources()
            .is_empty()
    );
}

#[test]
fn the_git_pickers_offer_lowercase_sources() {
    let sources = |name: &str| {
        Picker {
            name: name.to_string(),
            chord: Chord::parse("ctrl-g x").expect("a chord"),
            description: String::new(),
            command: PickerCommand::Builtin(name.to_string()),
            args: Vec::new(),
        }
        .sources()
    };

    assert_eq!(
        sources("status"),
        [
            "all",
            "conflicts",
            "staged",
            "unstaged",
            "untracked",
            "stashes",
            "tracked",
            "ignored"
        ]
    );
    assert_eq!(sources("history"), ["commits", "reflog", "changes"]);
    assert_eq!(sources("branches"), ["branches", "repos"]);
    assert!(sources("contents").is_empty());
    assert!(sources("git-config").is_empty());
}

#[test]
fn a_step_follows_the_query_when_its_builtin_reads_it_for_that_source() {
    assert!(follows_query("contents", None));
    assert!(follows_query("history", Some("changes")));
    assert!(!follows_query("history", Some("commits")));
    assert!(follows_query("rename-branch", None));
    assert!(!follows_query("checkout", None));
    assert!(!follows_query("files", Some("this folder")));
    assert!(!follows_query("my-script", None));
}

#[test]
fn the_steps_only_a_next_reaches_are_builtins_too() {
    for name in [
        "commit-files",
        "checkout",
        "stash-files",
        "rename-branch",
        "pull",
        "prune",
        "change-setting",
    ] {
        assert!(is_step_builtin(name), "{name}");
    }
    assert!(!is_step_builtin("files"));
}
