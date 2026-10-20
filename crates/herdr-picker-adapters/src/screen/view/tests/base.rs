use super::*;
use herdr_picker_domain::RowLook;

#[test]
fn every_row_is_painted_on_the_theme_to_the_popup_edge_so_the_terminal_shows_through_nowhere() {
    let workspaces = three_workspaces();
    let layout = layout().look(RowLook {
        leaders: false,
        stripes: true,
    });

    for width in 7..=40 {
        let drawn = render(&browsing(&workspaces, 0), &layout, width, 9);

        assert_eq!(drawn.lines().count(), 9);
        for row in drawn.lines() {
            assert!(
                [ON_BASE, ON_BAR, ON_STRIPE]
                    .iter()
                    .any(|background| row.starts_with(background)),
                "{row:?}"
            );
            assert!(row.ends_with(END), "{row:?}");
            assert_eq!(
                without_escape_sequences(row).chars().count(),
                width,
                "{row:?}"
            );
        }
    }
}
