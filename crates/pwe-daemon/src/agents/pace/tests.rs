use super::*;

#[test]
fn the_programs_are_read_at_most_once_per_refresh() {
    let mut pace = ReadingPace::every(15);
    let first = pace.wait_before_reading(100);
    pace.read_at(100);

    assert_eq!(first, 0);
    assert_eq!(pace.wait_before_reading(104), 11);
    assert_eq!(pace.wait_before_reading(115), 0);
    assert_eq!(pace.wait_before_reading(200), 0);
}
