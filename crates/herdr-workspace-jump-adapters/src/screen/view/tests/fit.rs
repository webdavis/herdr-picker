use super::*;

#[test]
fn the_columns_to_fit_are_the_widest_workspace_with_a_spaced_current_mark() {
    assert_eq!(columns_to_fit(&three_workspaces(), LEGEND), 23);
}

#[test]
fn a_legend_wider_than_every_workspace_sets_the_columns_to_fit() {
    let legend = Legend {
        search: "/ find",
        keys: "a legend wider than every workspace row",
    };

    assert_eq!(columns_to_fit(&three_workspaces(), legend), 40);
}

#[test]
fn at_the_columns_to_fit_every_label_and_its_current_mark_show_whole() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND);

    for (current, workspace) in workspaces.iter().enumerate() {
        let drawn = render(
            &browsing(&workspaces, current, Some(current)),
            LEGEND,
            width,
            0,
        );

        assert!(!drawn.contains(ELLIPSIS), "{drawn:?}");
        assert!(
            visible_rows(&drawn)
                .iter()
                .any(|row| row.contains(&workspace.label) && row.ends_with("current")),
            "{drawn:?}"
        );
    }
}

#[test]
fn one_column_fewer_leaves_the_widest_workspaces_current_mark_out() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND) - 1;

    let drawn = render(&browsing(&workspaces, 2, Some(2)), LEGEND, width, 0);

    assert!(!drawn.contains("current"), "{drawn:?}");
}
