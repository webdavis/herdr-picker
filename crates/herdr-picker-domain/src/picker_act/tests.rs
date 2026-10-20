use super::*;
use crate::picker_row::{Location, PickerCall};

fn row(enter: Action, keys: Vec<(&str, Action)>) -> PickerRow {
    PickerRow {
        label: "row".to_string(),
        preview: None,
        enter,
        keys: keys
            .into_iter()
            .map(|(name, action)| (Key::parse(name).expect("a key"), action))
            .collect(),
        group: None,
    }
}

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

fn edit(path: &str, line: Option<u32>) -> Action {
    Action::Edit(vec![Location {
        path: path.to_string(),
        line,
    }])
}

#[test]
fn one_row_does_what_its_key_says_and_nothing_on_a_key_it_does_not_claim() {
    let only = row(
        Action::Insert("main".to_string()),
        vec![("ctrl-o", Action::Open("https://example.com".to_string()))],
    );

    assert_eq!(
        act(&[&only], &Key::enter()),
        Ok(Action::Insert("main".to_string()))
    );
    assert_eq!(
        act(&[&only], &key("ctrl-o")),
        Ok(Action::Open("https://example.com".to_string()))
    );
    assert_eq!(act(&[&only], &key("alt-y")), Ok(Action::Nothing));
    assert_eq!(act(&[], &Key::enter()), Ok(Action::Nothing));
}

#[test]
fn marked_rows_insert_their_texts_joined_by_spaces_and_copy_them_one_per_line() {
    let first = row(
        Action::Insert("'a b'".to_string()),
        vec![("alt-y", Action::Copy("a b".to_string()))],
    );
    let second = row(
        Action::Insert("c".to_string()),
        vec![("alt-y", Action::Copy("c".to_string()))],
    );

    assert_eq!(
        act(&[&first, &second], &Key::enter()),
        Ok(Action::Insert("'a b' c".to_string()))
    );
    assert_eq!(
        act(&[&first, &second], &key("alt-y")),
        Ok(Action::Copy("a b\nc".to_string()))
    );
}

#[test]
fn marked_rows_edit_every_location_in_one_editor() {
    let first = row(edit("src/a.rs", Some(4)), Vec::new());
    let second = row(edit("src/b.rs", None), Vec::new());

    assert_eq!(
        act(&[&first, &second], &Key::enter()),
        Ok(Action::Edit(vec![
            Location {
                path: "src/a.rs".to_string(),
                line: Some(4)
            },
            Location {
                path: "src/b.rs".to_string(),
                line: None
            },
        ]))
    );
}

#[test]
fn actions_that_lead_one_place_refuse_more_than_one_marked_row() {
    let next = Action::Next(PickerCall {
        name: "files".to_string(),
        args: Vec::new(),
        state: None,
    });
    let confirm = Action::Confirm {
        question: "Delete?".to_string(),
        action: Box::new(Action::Run("rm x".to_string())),
    };
    for (action, sentence) in [
        (
            Action::Cd("/tmp".to_string()),
            "Select one row to change into",
        ),
        (Action::Run("date".to_string()), "Select one row to run"),
        (Action::Open("x".to_string()), "Select one row to open"),
        (next, "Select one row to go to its picker"),
        (confirm, "Select one row to confirm"),
    ] {
        let one = row(action.clone(), Vec::new());

        assert_eq!(act(&[&one], &Key::enter()), Ok(action.clone()));
        let refusal = act(&[&one, &one], &Key::enter()).expect_err("two rows are refused");
        assert_eq!(refusal.to_string(), sentence);
    }
}

#[test]
fn marked_rows_that_do_different_things_on_the_key_are_refused() {
    let insert = row(Action::Insert("a".to_string()), Vec::new());
    let copy = row(
        Action::Copy("b".to_string()),
        vec![("ctrl-o", Action::Open("b".to_string()))],
    );

    assert_eq!(
        act(&[&insert, &copy], &Key::enter()).map_err(|refusal| refusal.to_string()),
        Err("The marked rows do different things on enter".to_string())
    );
    assert_eq!(
        act(&[&insert, &copy], &key("ctrl-enter")).map_err(|refusal| refusal.to_string()),
        Ok(Action::Nothing)
    );
    assert_eq!(
        act(&[&copy, &insert], &key("ctrl-o")).map_err(|refusal| refusal.to_string()),
        Err("The marked rows do different things on ctrl-o".to_string())
    );
}

#[test]
fn reload_back_and_nothing_need_no_single_row() {
    for action in [Action::Reload, Action::Back, Action::Nothing] {
        let one = row(action.clone(), Vec::new());
        assert_eq!(act(&[&one, &one], &Key::enter()), Ok(action));
    }
}
