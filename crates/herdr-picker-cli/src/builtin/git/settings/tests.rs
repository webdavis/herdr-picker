use std::fs;

use herdr_picker_domain::{Action, Key, Location, PickerCall, PickerRow, Preview};

use super::line::{redact, setting_line};
use crate::builtin::git::fixture::{GitRepo, read};
use crate::builtin::tests::printed;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

fn configured(name: &str) -> GitRepo {
    let repo = GitRepo::new(name);
    repo.git(&["config", "pull.rebase", "false"]);
    repo.git(&["config", "my.token", "abc"]);
    repo.git(&[
        "config",
        "remote.origin.url",
        "https://me:pw@example.com/x.git",
    ]);
    repo.git(&[
        "config",
        "--add",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    repo.git(&[
        "config",
        "--add",
        "remote.origin.fetch",
        "+refs/tags/*:refs/tags/*",
    ]);
    repo
}

fn local(repo: &GitRepo) -> Vec<PickerRow> {
    let (_, rows) = read(&printed("git-config", &[], None, &repo.places()).expect("rows"));
    rows.into_iter()
        .filter(|row| row.group.as_deref() == Some("local"))
        .collect()
}

fn named<'a>(rows: &'a [PickerRow], name: &str) -> &'a PickerRow {
    rows.iter()
        .find(|row| {
            row.label.starts_with(&format!("{name} = "))
                || row.label.contains(&format!("{name} = "))
        })
        .unwrap_or_else(|| panic!("a row for {name}"))
}

#[test]
fn a_setting_shows_its_value_inserts_its_name_and_can_be_opened_changed_or_removed() {
    let repo = configured("settings-keys");
    let file = repo.path(".git/config");
    let line = fs::read_to_string(&file)
        .expect("the config")
        .lines()
        .position(|line| line.trim() == "rebase = false")
        .expect("the line") as u32
        + 1;

    let rows = local(&repo);
    let row = named(&rows, "pull.rebase");

    assert_eq!(row.label, format!("pull.rebase = {DIM}false{RESET}"));
    assert_eq!(row.enter, Action::Insert("pull.rebase".to_string()));
    assert_eq!(
        row.preview,
        Some(Preview::Text(
            "Value: false\nOrigin: file:.git/config".to_string()
        ))
    );
    assert_eq!(
        row.action(&key("ctrl-enter")),
        Some(&Action::Edit(vec![Location {
            path: file.clone(),
            line: Some(line)
        }]))
    );
    assert_eq!(
        row.action(&key("ctrl-v")),
        Some(&Action::Next(PickerCall {
            name: "change-setting".to_string(),
            args: vec![
                file.clone(),
                "pull.rebase".into(),
                "local".into(),
                "false".into()
            ],
            state: None,
            query: Some("false".to_string()),
        }))
    );
    assert_eq!(
        row.action(&key("ctrl-x")),
        Some(&Action::Confirm {
            question: "Remove pull.rebase = false from local?".to_string(),
            action: Box::new(Action::Run(format!(
                "git config --file {file} --unset pull.rebase"
            ))),
        })
    );
}

#[test]
fn secrets_are_hidden_and_a_setting_with_several_values_is_only_opened() {
    let repo = configured("settings-secret");

    let rows = local(&repo);

    assert_eq!(
        named(&rows, "my.token").label,
        format!("my.token = {DIM}[redacted]{RESET}")
    );
    assert_eq!(
        named(&rows, "remote.origin.url").label,
        format!("remote.origin.url = {DIM}https://[redacted]@example.com/x.git{RESET}")
    );
    let Some(Action::Next(change)) = named(&rows, "my.token").action(&key("ctrl-v")).cloned()
    else {
        panic!("a change step");
    };
    assert_eq!(change.query, None);
    let fetch = named(&rows, "remote.origin.fetch");
    assert_eq!(fetch.action(&key("ctrl-v")), None);
    assert_eq!(fetch.action(&key("ctrl-x")), None);
    let Some(Preview::Text(preview)) = &fetch.preview else {
        panic!("a text preview");
    };
    assert!(
        preview
            .ends_with("remote.origin.fetch holds several values; edit its file with Ctrl-Enter."),
        "{preview}"
    );
}

#[test]
fn a_value_a_later_file_sets_again_is_dimmed_as_overridden() {
    let repo = GitRepo::new("settings-overridden");
    repo.git(&["config", "pull.rebase", "false"]);
    repo.write(".git/extra", "[pull]\n\trebase = true\n");
    repo.git(&["config", "include.path", "extra"]);

    let rows = local(&repo);
    let first = rows
        .iter()
        .find(|row| row.label.contains("pull.rebase = false"))
        .expect("the first");

    assert_eq!(
        first.label,
        format!("{DIM}pull.rebase = false{RESET}  {DIM}overridden by local{RESET}")
    );
}

#[test]
fn a_settings_line_is_the_last_place_its_file_writes_it() {
    let repo = GitRepo::new("settings-line");
    repo.write(
        "c",
        "[pull]\n\trebase = false\n[remote \"origin\"]\n\turl = x\n[pull]\nrebase=true\n",
    );

    assert_eq!(setting_line(&repo.path("c"), "pull.rebase"), 6);
    assert_eq!(setting_line(&repo.path("c"), "remote.origin.url"), 4);
    assert_eq!(setting_line(&repo.path("c"), "nope.nothing"), 1);
}

#[test]
fn credentials_in_urls_and_query_strings_are_redacted() {
    assert_eq!(
        redact("https://me:pw@host/x?token=abc&b=1"),
        "https://[redacted]@host/x?token=[redacted]&b=1"
    );
    assert_eq!(redact("plain value"), "plain value");
}

#[test]
fn from_a_subfolder_the_repositorys_own_file_is_still_the_one_opened() {
    let repo = configured("settings-subfolder");
    repo.write("sub/a.txt", "x");
    let mut places = repo.places();
    places.cwd = repo.0.join("sub");

    let (_, rows) = read(&printed("git-config", &[], None, &places).expect("rows"));
    let rows: Vec<PickerRow> = rows
        .into_iter()
        .filter(|row| row.group.as_deref() == Some("local"))
        .collect();
    let row = named(&rows, "pull.rebase");

    let Some(Action::Edit(locations)) = row.action(&key("ctrl-enter")) else {
        panic!("an edit");
    };
    assert_eq!(locations[0].path, repo.path(".git/config"));
}
