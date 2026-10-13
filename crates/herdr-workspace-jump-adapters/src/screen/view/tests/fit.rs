use super::*;

#[test]
fn the_columns_to_fit_count_the_names_and_refusals_and_never_a_directory() {
    assert_eq!(columns_to_fit(&three_workspaces(), LEGEND), 20);
}

#[test]
fn a_legend_wider_than_every_workspace_sets_the_columns_to_fit() {
    let legend = Legend {
        search: "/ find",
        keys: "a legend wider than every workspace row and its directory",
    };

    assert_eq!(columns_to_fit(&three_workspaces(), legend), 58);
}

#[test]
fn at_the_columns_to_fit_every_label_shows_whole_and_the_directories_give_way() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND);

    let drawn = render(&browsing(&workspaces, 2), LEGEND, columns(), None, width, 0);

    for workspace in &workspaces {
        assert!(
            visible_rows(&drawn)
                .iter()
                .any(|row| row.contains(&format!("{} ", workspace.label))),
            "{drawn:?}"
        );
    }
}

#[test]
fn a_popup_wider_than_the_columns_to_fit_shows_the_directories_whole() {
    let workspaces = three_workspaces();

    let drawn = render(&browsing(&workspaces, 2), LEGEND, columns(), None, 31, 0);

    assert!(!drawn.contains(ELLIPSIS), "{drawn:?}");
    for workspace in &workspaces {
        assert!(
            visible_rows(&drawn)
                .iter()
                .any(|row| row.ends_with(&workspace.directory)),
            "{drawn:?}"
        );
    }
}

#[test]
fn one_column_fewer_cuts_the_widest_row() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND) - 1;

    let drawn = render(&browsing(&workspaces, 2), LEGEND, columns(), None, width, 0);

    assert!(drawn.contains(ELLIPSIS), "{drawn:?}");
}

#[test]
fn at_the_columns_to_fit_every_refusal_shows_whole() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND);

    for current in &workspaces {
        let drawn = render(
            &refused(&workspaces, current),
            LEGEND,
            columns(),
            None,
            width,
            0,
        );

        assert!(
            drawn.contains(&format!("already in {}\x1b", current.label)),
            "{drawn:?}"
        );
    }
}
