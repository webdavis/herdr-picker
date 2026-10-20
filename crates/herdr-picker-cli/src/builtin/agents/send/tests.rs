use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::builtin::tests::Tree;
use crate::cli_command::spawning;

fn asked<'a>(state: &'a str, query: &'a str) -> Asked<'a> {
    Asked {
        source: None,
        state: Some(state),
        query,
    }
}

#[test]
fn outside_herdr_the_rows_can_still_be_copied_or_asked_about() {
    let tree = Tree::new("send-outside", &[]);
    let state = r#"{"text":"a\nb","from":"Files"}"#;

    let rows = send_to(&asked(state, ""), &tree.places("home"));

    assert_eq!(
        rows.iter()
            .map(|row| (row.label.as_str(), row.enter.clone()))
            .collect::<Vec<_>>(),
        vec![
            ("copy instead", Action::Copy("a\nb".to_string())),
            (
                "quick answer",
                Action::Next(PickerCall {
                    name: "quick-answer".to_string(),
                    args: Vec::new(),
                    state: Some(state.to_string()),
                    query: None,
                })
            ),
        ]
    );
}

#[test]
fn a_question_is_asked_on_either_account_but_a_broccoli_pickers_rows_only_on_broccoli() {
    let labels = |state: &str| -> Vec<String> {
        quick_answer(&asked(state, " why? "))
            .into_iter()
            .map(|row| row.label)
            .collect()
    };

    assert_eq!(
        labels(r#"{"text":"x","from":"Files"}"#),
        vec!["Ask personal: why?", "Ask broccoli: why?"]
    );
    assert_eq!(
        labels(r#"{"text":"x","from":"Broccoli Tickets"}"#),
        vec!["Ask broccoli: why?"]
    );
    assert_eq!(
        quick_answer(&asked(r#"{"text":"x"}"#, ""))
            .iter()
            .map(|row| unstyled(&row.label))
            .collect::<Vec<_>>(),
        vec!["Type your question"]
    );
}

#[test]
fn the_quick_answer_runs_claude_print_mode_on_the_chosen_accounts_profile() {
    assert_eq!(
        quick_answer_argv("broccoli", "Why?", "row one"),
        [
            "clauth",
            "start",
            "broccoli",
            "-p",
            "Why?\n\nThe rows I picked:\nrow one"
        ]
    );
    assert_eq!(quick_answer_argv("personal", "Why?", "r")[2], "main");
}

#[test]
fn the_answer_shows_a_line_a_row_and_enter_copies_all_of_it() {
    let tree = Tree::new("send-answer", &[]);
    let fake = tree.0.join("clauth");
    {
        let _guard = spawning();
        fs::write(
            &fake,
            "#!/bin/sh\nprintf 'first line\\nasked: %s\\n' \"$4\"\n",
        )
        .expect("a fake");
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).expect("executable");
    }
    let argv = [
        fake.to_string_lossy().into_owned(),
        "start".into(),
        "main".into(),
        "-p".into(),
        "Why?".into(),
    ];

    let page = {
        let _guard = spawning();
        answer_page(&argv)
    };

    assert_eq!(
        page.iter()
            .map(|row| row.label.as_str())
            .collect::<Vec<_>>(),
        vec!["first line", "asked: Why?"]
    );
    assert_eq!(
        page[1].enter,
        Action::Copy("first line\nasked: Why?".to_string())
    );
}

#[test]
fn inside_herdr_each_running_agent_is_offered_first_and_enter_prompts_it() {
    let tree = Tree::new("send-inside", &[]);
    let server = crate::socket_server::SocketServer::start(super::super::tests::herdr_answers());
    let mut places = tree.places("home");
    places.herdr.socket = Some(server.path.clone());

    let rows = send_to(&asked(r#"{"text":"look"}"#, ""), &places);

    assert_eq!(
        rows.iter()
            .map(|row| unstyled(&row.label))
            .collect::<Vec<_>>(),
        vec![
            "✓ tidy  claude · dotfiles › one",
            "◐ build  claude · dotfiles › two",
            "● report  claude · dotfiles › three",
            "◉ review  claude · dotfiles › four",
            "copy instead",
            "quick answer",
        ]
    );
    assert_eq!(
        rows[3].enter,
        Action::Herdr(HerdrCall::Prompt {
            pane_id: "w1:p4".to_string(),
            text: "look".to_string(),
        })
    );
}
