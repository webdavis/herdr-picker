use super::*;
use crate::picker_row::Action;

fn row(label: &str, group: Option<&str>) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Insert(label.to_string()),
        keys: Vec::new(),
        group: group.map(str::to_string),
    }
}

fn two_groups() -> Vec<String> {
    vec!["local".to_string(), "remote".to_string()]
}

fn three_sources() -> Vec<String> {
    vec![
        "this folder".to_string(),
        "hidden".to_string(),
        "home".to_string(),
    ]
}

fn list(group: Option<&str>) -> View {
    View::List {
        group: group.map(str::to_string),
    }
}

fn streamed() -> View {
    View::Streamed { group: None }
}

fn chooser(group: Option<&str>) -> View {
    View::Chooser {
        group: group.map(str::to_string),
    }
}

#[test]
fn groups_are_the_distinct_groups_the_rows_carry_in_name_order() {
    let rows = [
        row("main", Some("remote")),
        row("dev", Some("local")),
        row("loose", None),
        row("next", Some("remote")),
    ];

    assert_eq!(groups(&rows), two_groups());
    assert_eq!(groups(&[row("main", None)]), Vec::<String>::new());
}

#[test]
fn the_chooser_offers_all_first_then_each_group() {
    assert_eq!(
        chooser_entries(&[], &two_groups()),
        vec!["all", "local", "remote"]
    );
}

#[test]
fn a_list_shows_every_row_or_only_its_groups_rows_with_their_indexes() {
    let rows = [
        row("main", Some("remote")),
        row("dev", Some("local")),
        row("next", Some("remote")),
    ];
    let labels = |view: &View| -> Vec<(usize, String)> {
        shown(&rows, view)
            .into_iter()
            .map(|(index, row)| (index, row.label.clone()))
            .collect()
    };

    assert_eq!(labels(&list(None)).len(), 3);
    assert_eq!(
        labels(&list(Some("remote"))),
        vec![(0, "main".to_string()), (2, "next".to_string())]
    );
    assert_eq!(labels(&streamed()).len(), 3);
}

#[test]
fn once_grouped_rows_load_the_list_is_redrawn_with_its_group_column() {
    assert_eq!(
        press(&streamed(), ViewKey::Loaded, &[], &two_groups()),
        (list(None), ViewResponse::Redraw)
    );
    assert_eq!(
        press(&streamed(), ViewKey::Loaded, &[], &[]),
        (streamed(), ViewResponse::Stay)
    );
    assert_eq!(
        press(&list(Some("local")), ViewKey::Loaded, &[], &two_groups()),
        (list(Some("local")), ViewResponse::Stay)
    );
}

#[test]
fn choose_opens_the_chooser_only_over_a_grouped_list() {
    assert_eq!(
        press(&list(Some("local")), ViewKey::Choose, &[], &two_groups()),
        (chooser(Some("local")), ViewResponse::OpenChooser)
    );
    assert_eq!(
        press(&streamed(), ViewKey::Choose, &[], &[]),
        (streamed(), ViewResponse::Stay)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Choose, &[], &two_groups()),
        (chooser(None), ViewResponse::Stay)
    );
}

#[test]
fn enter_in_the_chooser_filters_the_list_to_the_picked_entry() {
    assert_eq!(
        press(
            &chooser(Some("local")),
            ViewKey::Enter(Some(0)),
            &[],
            &two_groups()
        ),
        (list(None), ViewResponse::CloseChooser)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(2)), &[], &two_groups()),
        (list(Some("remote")), ViewResponse::CloseChooser)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(3)), &[], &two_groups()),
        (chooser(None), ViewResponse::Stay)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(None), &[], &two_groups()),
        (chooser(None), ViewResponse::Stay)
    );
}

#[test]
fn back_in_the_chooser_returns_to_the_list_it_came_from() {
    assert_eq!(
        press(&chooser(Some("local")), ViewKey::Back, &[], &two_groups()),
        (list(Some("local")), ViewResponse::CloseChooser)
    );
}

#[test]
fn rows_streamed_back_into_a_step_filter_to_the_group_it_was_left_on() {
    let returning = |group: &str| View::Streamed {
        group: Some(group.to_string()),
    };

    assert_eq!(
        press(&returning("remote"), ViewKey::Loaded, &[], &two_groups()),
        (list(Some("remote")), ViewResponse::Redraw)
    );
    assert_eq!(
        press(&returning("gone"), ViewKey::Loaded, &[], &two_groups()),
        (list(None), ViewResponse::Redraw)
    );
}

#[test]
fn outside_the_chooser_a_row_key_accepts_and_back_leaves_the_step() {
    for view in [streamed(), list(Some("local"))] {
        for key in [ViewKey::Enter(Some(1)), ViewKey::Other] {
            assert_eq!(
                press(&view, key, &[], &two_groups()),
                (view.clone(), ViewResponse::Accept)
            );
        }
        assert_eq!(
            press(&view, ViewKey::Back, &[], &two_groups()),
            (view.clone(), ViewResponse::Back)
        );
    }
}

#[test]
fn in_the_chooser_a_row_key_besides_enter_does_nothing() {
    assert_eq!(
        press(&chooser(None), ViewKey::Other, &[], &two_groups()),
        (chooser(None), ViewResponse::Stay)
    );
}

#[test]
fn the_chooser_offers_the_sources_then_all_and_each_group_when_rows_have_groups() {
    assert_eq!(
        chooser_entries(&three_sources(), &[]),
        vec!["this folder", "hidden", "home"]
    );
    assert_eq!(
        chooser_entries(&three_sources(), &two_groups()),
        vec!["this folder", "hidden", "home", "all", "local", "remote"]
    );
}

#[test]
fn with_sources_choose_opens_the_chooser_while_rows_still_stream() {
    assert_eq!(
        press(&streamed(), ViewKey::Choose, &three_sources(), &[]),
        (chooser(None), ViewResponse::OpenChooser)
    );
}

#[test]
fn enter_on_a_source_switches_to_it_and_on_a_group_filters_to_it() {
    let (sources, groups) = (three_sources(), two_groups());

    assert_eq!(
        press(
            &chooser(Some("local")),
            ViewKey::Enter(Some(1)),
            &sources,
            &groups
        ),
        (streamed(), ViewResponse::Switch("hidden".to_string()))
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(3)), &sources, &groups),
        (list(None), ViewResponse::CloseChooser)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(5)), &sources, &groups),
        (list(Some("remote")), ViewResponse::CloseChooser)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(3)), &sources, &[]),
        (chooser(None), ViewResponse::Stay)
    );
}
