use super::*;

fn call(name: &str) -> PickerCall {
    PickerCall {
        name: name.to_string(),
        args: Vec::new(),
        state: None,
    }
}

fn spot(query: &str, group: Option<&str>) -> Spot {
    Spot {
        query: query.to_string(),
        group: group.map(str::to_string),
        source: None,
    }
}

fn confirm(question: &str, action: Action) -> Action {
    Action::Confirm {
        question: question.to_string(),
        action: Box::new(action),
    }
}

#[test]
fn next_pushes_a_step_and_back_restores_the_query_and_group_left_behind() {
    let mut trail = Trail::new(call("git-branches"));

    assert_eq!(
        trail.advance(Action::Next(call("files")), spot("ma", Some("local"))),
        Turn::Show
    );
    assert_eq!(trail.titles(), vec!["Git Branches", "Files"]);
    assert_eq!(trail.current().spot, Spot::default());

    assert_eq!(trail.advance(Action::Back, spot("x", None)), Turn::Show);
    assert_eq!(trail.titles(), vec!["Git Branches"]);
    assert_eq!(trail.current().spot, spot("ma", Some("local")));
}

#[test]
fn back_on_the_first_step_closes() {
    let mut trail = Trail::new(call("files"));

    assert_eq!(trail.advance(Action::Back, Spot::default()), Turn::Close);
}

#[test]
fn reload_shows_the_same_step_again_where_the_user_was() {
    let mut trail = Trail::new(call("files"));

    assert_eq!(trail.advance(Action::Reload, spot("src", None)), Turn::Show);
    assert_eq!(trail.titles(), vec!["Files"]);
    assert_eq!(trail.current().spot, spot("src", None));
}

#[test]
fn actions_for_the_shell_leave_the_picker() {
    let mut trail = Trail::new(call("files"));

    assert_eq!(
        trail.advance(Action::Cd("/tmp".to_string()), Spot::default()),
        Turn::Leave(Action::Cd("/tmp".to_string()))
    );
}

#[test]
fn a_confirm_asks_its_question_as_a_step_and_no_returns() {
    let mut trail = Trail::new(call("files"));
    let remove = Action::Run("rm notes.txt".to_string());

    trail.advance(
        confirm("Delete notes.txt?", remove.clone()),
        spot("no", None),
    );

    assert_eq!(trail.titles(), vec!["Files", "Delete notes.txt?"]);
    assert_eq!(
        trail.current().page,
        Page::Confirm {
            question: "Delete notes.txt?".to_string(),
            action: remove,
        }
    );
    assert_eq!(trail.advance(Action::Back, Spot::default()), Turn::Show);
    assert_eq!(trail.titles(), vec!["Files"]);
    assert_eq!(trail.current().spot, spot("no", None));
}

#[test]
fn an_answered_confirm_leaves_the_trail_before_its_action_runs() {
    let mut trail = Trail::new(call("files"));
    trail.advance(
        confirm("Open the log?", Action::Next(call("log"))),
        spot("q", None),
    );

    assert_eq!(
        trail.advance(Action::Next(call("log")), Spot::default()),
        Turn::Show
    );
    assert_eq!(trail.titles(), vec!["Files", "Log"]);

    trail.advance(Action::Back, Spot::default());
    assert_eq!(trail.current().spot, spot("q", None));
}

#[test]
fn a_source_action_shows_the_step_again_from_that_source_keeping_the_query() {
    let mut trail = Trail::new(call("files"));

    assert_eq!(
        trail.advance(
            Action::Source("hidden".to_string()),
            spot("notes", Some("src"))
        ),
        Turn::Show
    );
    assert_eq!(trail.titles(), vec!["Files"]);
    assert_eq!(
        trail.current().spot,
        Spot {
            query: "notes".to_string(),
            group: None,
            source: Some("hidden".to_string()),
        }
    );
}

#[test]
fn back_restores_the_source_a_step_was_left_on() {
    let mut trail = Trail::new(call("files"));
    trail.advance(Action::Source("home".to_string()), Spot::default());

    trail.advance(Action::Next(call("log")), trail.current().spot.clone());
    trail.advance(Action::Back, Spot::default());

    assert_eq!(trail.current().spot.source, Some("home".to_string()));
}
