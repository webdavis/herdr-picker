use super::*;

#[test]
fn the_columns_to_fit_are_the_widest_refusal_or_the_legend_and_never_a_directory() {
    assert_eq!(layout().columns_to_fit(), 20);
}

#[test]
fn a_legend_wider_than_every_workspace_sets_the_columns_to_fit() {
    let legend = Legend {
        search: "/ find",
        keys: "a legend wider than every workspace row and its directory",
    };

    assert_eq!(
        Layout::new(legend, &three_workspaces()).columns_to_fit(),
        58
    );
}

#[test]
fn at_the_columns_to_fit_every_label_shows_whole_and_the_directories_give_way() {
    let workspaces = three_workspaces();
    let width = layout().columns_to_fit();

    let drawn = render(&browsing(&workspaces, 2), &layout(), width, 0);

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

    let drawn = render(&browsing(&workspaces, 2), &layout(), 31, 0);

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
fn one_column_fewer_cuts_the_widest_refusal() {
    let workspaces = three_workspaces();
    let width = layout().columns_to_fit() - 1;

    let drawn = render(&refused(&workspaces, &workspaces[2]), &layout(), width, 0);

    assert_eq!(
        drawn.lines().rev().nth(1),
        Some("\x1b[38;2;243;139;168m already in dotfil\u{2026}\x1b[0m")
    );
}

#[test]
fn at_the_columns_to_fit_every_refusal_shows_whole() {
    let workspaces = three_workspaces();
    let width = layout().columns_to_fit();

    for current in &workspaces {
        let drawn = render(&refused(&workspaces, current), &layout(), width, 0);

        assert!(
            drawn.contains(&format!("already in {}\x1b", current.label)),
            "{drawn:?}"
        );
    }
}
