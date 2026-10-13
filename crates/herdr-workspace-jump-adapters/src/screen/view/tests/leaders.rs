use super::*;

#[test]
fn faint_leaders_run_from_each_name_to_the_directory_column_with_a_space_either_side() {
    let workspaces = three_workspaces();

    let drawn = render(&browsing(&workspaces, 0), &layout(), 0, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[1;34m   h\x1b[22;39m  homelab\x1b[38;5;239m ·· \x1b[22;39m\x1b[2m/opt/homelab\x1b[22;39m"
        )
    );
    assert_eq!(
        visible_rows(&drawn)[1..4],
        [
            "i  Ivy ······ /opt/Ivy",
            "h  homelab ·· /opt/homelab",
            "f  dotfiles · /opt/dotfiles"
        ]
    );
}

#[test]
fn the_directory_column_is_set_by_the_widest_name_offered_not_by_a_hidden_one() {
    let workspaces = three_workspaces();
    let layout = layout().hiding(Some("dotfiles"));

    let drawn = render(&browsing(&workspaces[..2], 0), &layout, 0, 0);

    assert_eq!(
        visible_rows(&drawn)[1..3],
        ["i  Ivy ····· /opt/Ivy", "h  homelab · /opt/homelab"]
    );
}
