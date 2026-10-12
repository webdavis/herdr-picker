use super::*;

fn size(columns: usize, rows: usize) -> Size {
    Size { columns, rows }
}

#[test]
fn the_popup_takes_74_percent_of_the_window_wide_and_62_percent_tall() {
    assert_eq!(popup_size(size(175, 65), 40), size(129, 40));
}

#[test]
fn the_shares_round_down_as_herdrs_own_percentages_do() {
    assert_eq!(popup_size(size(58, 29), 30), size(42, 17));
}

#[test]
fn a_row_wider_than_that_share_widens_the_popup_by_the_row_and_herdrs_border() {
    assert_eq!(popup_size(size(58, 29), 40), size(43, 17));
}

#[test]
fn the_popup_never_grows_past_the_window() {
    assert_eq!(popup_size(size(50, 20), 60), size(50, 12));
}
