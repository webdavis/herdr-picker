use super::*;

const UNTIL: u64 = 1790000106;

#[test]
fn the_notice_program_ends_at_its_time() {
    assert_eq!(
        notice_step(UNTIL, NoticeEvent::TimeIs(UNTIL)),
        NoticeStep::End
    );
    assert_eq!(
        notice_step(UNTIL, NoticeEvent::TimeIs(UNTIL + 30)),
        NoticeStep::End
    );
}

#[test]
fn before_its_time_it_waits_for_the_seconds_left() {
    assert_eq!(
        notice_step(UNTIL, NoticeEvent::TimeIs(UNTIL - 4)),
        NoticeStep::Wait { seconds: 4 }
    );
}

#[test]
fn a_key_or_a_click_ends_it_early() {
    assert_eq!(notice_step(UNTIL, NoticeEvent::KeyOrClick), NoticeStep::End);
}

#[test]
fn a_resize_paints_the_pane_again() {
    assert_eq!(notice_step(UNTIL, NoticeEvent::Resized), NoticeStep::Paint);
}
