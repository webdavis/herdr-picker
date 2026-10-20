use super::*;

#[test]
fn a_moment_shows_in_local_time_to_the_minute() {
    assert_eq!(stamp(0, 0), "1970-01-01 00:00");
    assert_eq!(stamp(1_792_404_464, -6 * 3600), "2026-10-19 04:07");
    assert_eq!(stamp(951_782_400, 0), "2000-02-29 00:00");
}

#[test]
fn iso_times_with_a_zone_or_fraction_name_one_moment() {
    assert_eq!(parse_iso("2026-10-19T10:07:44Z"), Some(1_792_404_464));
    assert_eq!(
        parse_iso("2026-10-19T12:07:44.512+02:00"),
        Some(1_792_404_464)
    );
    assert_eq!(parse_iso("2026-10-19T04:07:44-0600"), Some(1_792_404_464));
    assert_eq!(parse_iso("yesterday"), None);
}

#[test]
fn a_zone_offset_reads_as_seconds_east() {
    assert_eq!(offset("-0600"), Some(-21_600));
    assert_eq!(offset("+0530"), Some(19_800));
    assert_eq!(offset("UTC"), None);
}
