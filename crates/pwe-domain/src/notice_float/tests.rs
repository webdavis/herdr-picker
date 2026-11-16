use super::*;

const WINDOW: Size = Size {
    columns: 100,
    rows: 50,
};

#[test]
fn a_float_is_one_title_bar_high_and_as_wide_as_its_line() {
    let place = notice_place(WINDOW, Corner::TopRight, 0, "fourteen chars");

    assert_eq!(place.bounds, [0.8, 0.0, 0.2, 0.05]);
    assert_eq!(
        place.size,
        Size {
            columns: 20,
            rows: 1
        }
    );
}

#[test]
fn each_slot_from_a_top_corner_sits_one_bar_lower() {
    let place = notice_place(WINDOW, Corner::TopLeft, 2, "fourteen chars");

    assert_eq!(place.bounds, [0.0, 0.1, 0.2, 0.05]);
}

#[test]
fn each_slot_from_a_bottom_corner_sits_one_bar_higher() {
    let place = notice_place(WINDOW, Corner::BottomRight, 1, "fourteen chars");

    assert_eq!(place.bounds[1], 1.0 - 0.05 - 0.05);
}

#[test]
fn a_long_line_is_capped_at_half_the_window() {
    let place = notice_place(WINDOW, Corner::TopRight, 0, &"x".repeat(128));

    assert_eq!(place.bounds[2], 0.5);
    assert_eq!(place.size.columns, 50);
}

#[test]
fn a_slot_past_the_window_stays_inside_it() {
    let place = notice_place(WINDOW, Corner::TopRight, 40, "line");

    assert_eq!(place.bounds[1], 0.95);
}
