use pwe_domain::Level;

use super::*;

#[test]
fn a_shown_level_and_a_time_are_read() {
    assert!(matches!(
        notice_options("needs-you", "1790000106"),
        Ok((Level::NeedsYou, 1790000106))
    ));
    assert!(matches!(
        notice_options("finished", "1790000106"),
        Ok((Level::Finished, 1790000106))
    ));
}

#[test]
fn a_quiet_or_unknown_level_or_a_bad_time_is_a_usage_error() {
    assert!(matches!(
        notice_options("quiet", "1"),
        Err(CommandError::Usage)
    ));
    assert!(matches!(
        notice_options("loud", "1"),
        Err(CommandError::Usage)
    ));
    assert!(matches!(
        notice_options("finished", "soon"),
        Err(CommandError::Usage)
    ));
}
