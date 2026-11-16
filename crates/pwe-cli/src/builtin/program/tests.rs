use super::*;

#[test]
fn a_program_that_ends_in_time_gives_what_it_printed() {
    let done = ran(
        &["sh", "-c", "cat; echo err >&2"],
        Some("typed"),
        Duration::from_secs(5),
    )
    .expect("it runs");

    assert_eq!(done.stdout, "typed");
    assert_eq!(done.stderr, "err\n");
    assert!(done.succeeded);
}

#[test]
fn a_program_past_its_deadline_is_stopped_with_what_it_printed_so_far() {
    let started = Instant::now();
    let done = ran(
        &["sh", "-c", "echo early; exec sleep 5"],
        None,
        Duration::from_millis(300),
    )
    .expect("it runs");

    assert_eq!(done.stdout, "early\n");
    assert!(!done.succeeded);
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[test]
fn run_refuses_a_failing_or_missing_program_with_the_reason() {
    assert_eq!(
        run(
            &["sh", "-c", "echo nope >&2; exit 3"],
            Duration::from_secs(5)
        ),
        Err("sh failed: nope".to_string())
    );
    assert!(
        run(&["no-such-program-hwj"], Duration::from_secs(1))
            .is_err_and(|why| why.starts_with("no-such-program-hwj: "))
    );
}
