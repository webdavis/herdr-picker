use super::*;

#[test]
fn the_columns_to_fit_leave_room_to_refuse_the_widest_workspace() {
    assert_eq!(columns_to_fit(&three_workspaces(), LEGEND), 20);
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
fn at_the_columns_to_fit_every_label_shows_whole() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND);

    let drawn = render(&browsing(&workspaces, 2), LEGEND, width, 0);

    assert!(!drawn.contains(ELLIPSIS), "{drawn:?}");
    for workspace in &workspaces {
        assert!(
            visible_rows(&drawn)
                .iter()
                .any(|row| row.ends_with(&workspace.label)),
            "{drawn:?}"
        );
    }
}

#[test]
fn one_column_fewer_cuts_the_widest_refusal() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND) - 1;

    let drawn = render(&refused(&workspaces, &workspaces[2]), LEGEND, width, 0);

    assert!(drawn.contains(ELLIPSIS), "{drawn:?}");
}

#[test]
fn at_the_columns_to_fit_every_refusal_shows_whole() {
    let workspaces = three_workspaces();
    let width = columns_to_fit(&workspaces, LEGEND);

    for current in &workspaces {
        let drawn = render(&refused(&workspaces, current), LEGEND, width, 0);

        assert!(!drawn.contains(ELLIPSIS), "{drawn:?}");
    }
}
