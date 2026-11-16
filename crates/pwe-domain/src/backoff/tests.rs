use super::*;

#[test]
fn backoff_doubles_to_thirty_seconds_and_resets_on_success() {
    let mut backoff = Backoff::default();

    let waits: Vec<u64> = (0..7).map(|_| backoff.wait_after_failure()).collect();
    backoff.succeeded();

    assert_eq!(waits, vec![1, 2, 4, 8, 16, 30, 30]);
    assert_eq!(backoff.wait_after_failure(), 1);
}
