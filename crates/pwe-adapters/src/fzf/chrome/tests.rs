use pwe_domain::Action;

use super::*;

const HELPER: Helper = Helper {
    program: "'/bin/pwe'",
    cache: "'/state/rows'",
};

fn row(label: &str, group: Option<&str>) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Insert(label.to_string()),
        keys: Vec::new(),
        group: group.map(str::to_string),
    }
}

#[test]
fn rows_without_groups_are_index_tab_label() {
    assert_eq!(
        list_lines(
            &[row("alpha", None), row("beta", None)],
            &View::Streamed { group: None }
        ),
        vec!["0\talpha", "1\tbeta"]
    );
}

#[test]
fn grouped_rows_show_the_group_as_a_dim_column_as_wide_as_the_longest() {
    let rows = [
        row("main", Some("remote")),
        row("dev", Some("local")),
        row("loose", None),
    ];

    assert_eq!(
        list_lines(&rows, &View::List { group: None }),
        vec![
            "0\t\x1b[2mremote\x1b[22m  main",
            "1\t\x1b[2mlocal \x1b[22m  dev",
            "2\t\x1b[2m      \x1b[22m  loose",
        ]
    );
    assert_eq!(
        list_lines(
            &rows,
            &View::List {
                group: Some("local".to_string())
            }
        ),
        vec!["1\t\x1b[2mlocal\x1b[22m  dev"]
    );
}

#[test]
fn the_chooser_lists_its_entries_by_index() {
    assert_eq!(
        chooser_lines(&["hidden", "all", "remote"]),
        vec!["0\thidden", "1\tall", "2\tremote"]
    );
}

#[test]
fn each_view_response_becomes_the_fzf_actions_that_draw_it() {
    let actions = |response| finder_actions(response, HELPER, false, false);

    assert_eq!(
        actions(ViewResponse::Switch("hidden".to_string())),
        "print(source)+accept"
    );

    assert_eq!(
        actions(ViewResponse::Redraw),
        "reload('/bin/pwe' rows '/state/rows')"
    );
    assert_eq!(
        actions(ViewResponse::OpenChooser),
        "reload('/bin/pwe' rows '/state/rows')+clear-query+hide-preview"
    );
    assert_eq!(
        actions(ViewResponse::CloseChooser),
        "reload('/bin/pwe' rows '/state/rows')+clear-query+show-preview"
    );
    assert_eq!(actions(ViewResponse::Accept), "accept");
    assert_eq!(actions(ViewResponse::Back), "print(esc)+accept");
    assert_eq!(actions(ViewResponse::Stay), "");
}

#[test]
fn a_live_steps_chooser_searches_what_is_typed_and_closing_it_stops_searching_again() {
    assert_eq!(
        finder_actions(ViewResponse::OpenChooser, HELPER, false, true),
        "reload('/bin/pwe' rows '/state/rows')+clear-query+hide-preview+enable-search"
    );
    assert_eq!(
        finder_actions(ViewResponse::CloseChooser, HELPER, false, true),
        "reload('/bin/pwe' rows '/state/rows')+clear-query+show-preview+disable-search"
    );
}

#[test]
fn a_failed_run_also_asks_pwe_for_the_footer() {
    assert_eq!(
        finder_actions(ViewResponse::Stay, HELPER, true, false),
        "transform-footer('/bin/pwe' footer '/state/rows')"
    );
    assert_eq!(
        finder_actions(ViewResponse::Redraw, HELPER, true, false),
        "reload('/bin/pwe' rows '/state/rows')+transform-footer('/bin/pwe' footer '/state/rows')"
    );
}

#[test]
fn a_key_that_leaves_closes_fzf_printing_its_name() {
    assert_eq!(leave("ctrl-o"), "print(ctrl-o)+accept");
    assert_eq!(
        change_footer("Select one row to run"),
        "change-footer:Select one row to run"
    );
}
