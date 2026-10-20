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

fn sources() -> Vec<String> {
    vec!["local".to_string(), "remote".to_string()]
}

fn list(group: Option<&str>) -> View {
    View::List {
        group: group.map(str::to_string),
    }
}

fn streamed() -> View {
    View::Streamed { group: None }
}

fn titles(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
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

    assert_eq!(groups(&rows), sources());
    assert_eq!(groups(&[row("main", None)]), Vec::<String>::new());
}

#[test]
fn the_chooser_offers_all_first_then_each_group() {
    assert_eq!(chooser_entries(&sources()), vec!["all", "local", "remote"]);
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
fn a_title_is_the_picker_name_in_words() {
    assert_eq!(title("git-branches"), "Git Branches");
    assert_eq!(title("my_files"), "My Files");
}

#[test]
fn the_header_is_the_trail_of_titles_then_the_source_when_the_rows_have_groups() {
    let branches = titles(&["Git Branches"]);
    assert_eq!(header(&branches, &streamed(), &[]), "Git Branches");
    assert_eq!(
        header(&branches, &streamed(), &sources()),
        "Git Branches | Source: all"
    );
    assert_eq!(
        header(&branches, &list(Some("remote")), &sources()),
        "Git Branches | Source: remote"
    );
    assert_eq!(
        header(&titles(&["My Files"]), &chooser(None), &sources()),
        "My Files › Choose source"
    );
    assert_eq!(
        header(&titles(&["Git Branches", "Files"]), &list(None), &sources()),
        "Git Branches › Files | Source: all"
    );
}

#[test]
fn once_grouped_rows_load_the_list_is_redrawn_with_its_group_column() {
    assert_eq!(
        press(&streamed(), ViewKey::Loaded, &sources()),
        (list(None), ViewResponse::Redraw)
    );
    assert_eq!(
        press(&streamed(), ViewKey::Loaded, &[]),
        (streamed(), ViewResponse::Stay)
    );
    assert_eq!(
        press(&list(Some("local")), ViewKey::Loaded, &sources()),
        (list(Some("local")), ViewResponse::Stay)
    );
}

#[test]
fn choose_opens_the_chooser_only_over_a_grouped_list() {
    assert_eq!(
        press(&list(Some("local")), ViewKey::Choose, &sources()),
        (chooser(Some("local")), ViewResponse::OpenChooser)
    );
    assert_eq!(
        press(&streamed(), ViewKey::Choose, &[]),
        (streamed(), ViewResponse::Stay)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Choose, &sources()),
        (chooser(None), ViewResponse::Stay)
    );
}

#[test]
fn enter_in_the_chooser_filters_the_list_to_the_picked_entry() {
    assert_eq!(
        press(&chooser(Some("local")), ViewKey::Enter(Some(0)), &sources()),
        (list(None), ViewResponse::CloseChooser)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(2)), &sources()),
        (list(Some("remote")), ViewResponse::CloseChooser)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(Some(3)), &sources()),
        (chooser(None), ViewResponse::Stay)
    );
    assert_eq!(
        press(&chooser(None), ViewKey::Enter(None), &sources()),
        (chooser(None), ViewResponse::Stay)
    );
}

#[test]
fn back_in_the_chooser_returns_to_the_list_it_came_from() {
    assert_eq!(
        press(&chooser(Some("local")), ViewKey::Back, &sources()),
        (list(Some("local")), ViewResponse::CloseChooser)
    );
}

#[test]
fn rows_streamed_back_into_a_step_filter_to_the_group_it_was_left_on() {
    let returning = |group: &str| View::Streamed {
        group: Some(group.to_string()),
    };

    assert_eq!(
        press(&returning("remote"), ViewKey::Loaded, &sources()),
        (list(Some("remote")), ViewResponse::Redraw)
    );
    assert_eq!(
        press(&returning("gone"), ViewKey::Loaded, &sources()),
        (list(None), ViewResponse::Redraw)
    );
}

#[test]
fn outside_the_chooser_a_row_key_accepts_and_back_leaves_the_step() {
    for view in [streamed(), list(Some("local"))] {
        for key in [ViewKey::Enter(Some(1)), ViewKey::Other] {
            assert_eq!(
                press(&view, key, &sources()),
                (view.clone(), ViewResponse::Accept)
            );
        }
        assert_eq!(
            press(&view, ViewKey::Back, &sources()),
            (view.clone(), ViewResponse::Back)
        );
    }
}

#[test]
fn in_the_chooser_a_row_key_besides_enter_does_nothing() {
    assert_eq!(
        press(&chooser(None), ViewKey::Other, &sources()),
        (chooser(None), ViewResponse::Stay)
    );
}
