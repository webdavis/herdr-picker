use herdr_picker_domain::Action;

use super::*;

const HELPER: Helper = Helper {
    program: "'/bin/herdr-picker'",
    cache: "'/state/rows'",
};

fn row(label: &str, group: Option<&str>) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Insert(label.to_string()),
        group: group.map(str::to_string),
    }
}

#[test]
fn rows_without_groups_are_index_tab_label() {
    assert_eq!(
        list_lines(&[row("alpha", None), row("beta", None)], &View::Streamed),
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
fn the_chooser_lists_all_then_each_group_by_index() {
    assert_eq!(
        chooser_lines(&["local".to_string(), "remote".to_string()]),
        vec!["0\tall", "1\tlocal", "2\tremote"]
    );
}

#[test]
fn each_view_response_becomes_the_fzf_actions_that_draw_it() {
    let actions = |response| finder_actions(response, HELPER, false);

    assert_eq!(
        actions(ViewResponse::Redraw),
        "reload('/bin/herdr-picker' rows '/state/rows')"
    );
    assert_eq!(
        actions(ViewResponse::OpenChooser),
        "reload('/bin/herdr-picker' rows '/state/rows')+clear-query+hide-preview"
    );
    assert_eq!(
        actions(ViewResponse::CloseChooser),
        "reload('/bin/herdr-picker' rows '/state/rows')+clear-query+show-preview"
    );
    assert_eq!(actions(ViewResponse::Accept), "accept");
    assert_eq!(actions(ViewResponse::Abort), "abort");
    assert_eq!(actions(ViewResponse::Stay), "");
}

#[test]
fn a_failed_run_also_asks_herdr_picker_for_the_footer() {
    assert_eq!(
        finder_actions(ViewResponse::Stay, HELPER, true),
        "transform-footer('/bin/herdr-picker' footer '/state/rows')"
    );
    assert_eq!(
        finder_actions(ViewResponse::Redraw, HELPER, true),
        "reload('/bin/herdr-picker' rows '/state/rows')+transform-footer('/bin/herdr-picker' footer '/state/rows')"
    );
}

#[test]
fn an_error_footer_is_red_and_wrapped_on_words_to_the_width() {
    assert_eq!(
        error_footer(
            "line 2 is not a picker row: expected value",
            16,
            Theme::CatppuccinMocha
        ),
        "\x1b[38;2;243;139;168mline 2 is not a\x1b[0m\n\x1b[38;2;243;139;168mpicker row:\x1b[0m\n\x1b[38;2;243;139;168mexpected value\x1b[0m"
    );
}
