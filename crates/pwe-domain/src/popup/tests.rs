use super::*;

fn size(columns: usize, rows: usize) -> Size {
    Size { columns, rows }
}

#[test]
fn the_popup_takes_40_percent_of_the_window_each_way() {
    assert_eq!(popup_size(size(175, 65), 40), size(70, 26));
}

#[test]
fn the_shares_round_down_as_herdrs_own_percentages_do() {
    assert_eq!(popup_size(size(177, 67), 40), size(70, 26));
}

#[test]
fn a_small_window_still_gets_a_popup_of_60_columns_by_14_rows() {
    assert_eq!(popup_size(size(100, 40), 40), size(60, 16));
}

#[test]
fn a_large_window_gets_a_popup_of_at_most_100_columns_by_30_rows() {
    assert_eq!(popup_size(size(300, 90), 40), size(100, 30));
}

#[test]
fn the_smallest_popup_is_still_cut_to_the_window() {
    assert_eq!(popup_size(size(58, 29), 40), size(58, 14));
}

#[test]
fn a_row_wider_than_the_largest_popup_widens_it_by_the_row_and_herdrs_border() {
    assert_eq!(popup_size(size(300, 90), 120), size(123, 30));
}

#[test]
fn the_popup_never_grows_past_the_window() {
    assert_eq!(popup_size(size(50, 10), 60), size(50, 10));
}

#[test]
fn a_picker_with_a_preview_takes_80_percent_of_the_window_each_way() {
    assert_eq!(preview_popup_size(size(175, 50)), size(140, 40));
}

#[test]
fn a_small_window_still_gets_a_preview_popup_of_60_columns_by_14_rows() {
    assert_eq!(preview_popup_size(size(70, 15)), size(60, 14));
}

#[test]
fn a_large_window_gets_a_preview_popup_of_at_most_180_columns_by_40_rows() {
    assert_eq!(preview_popup_size(size(300, 90)), size(180, 40));
}

#[test]
fn the_preview_popup_never_grows_past_the_window() {
    assert_eq!(preview_popup_size(size(50, 10)), size(50, 10));
}
