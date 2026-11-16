use super::*;

#[test]
fn a_job_every_two_seconds_is_due_once_each_two_seconds() {
    let mut timers = Timers::default();
    timers.every("refresh", 2, 100);

    let due_by_second: Vec<Vec<String>> = (100..=106).map(|now| timers.due(now)).collect();

    let refresh = vec!["refresh".to_string()];
    assert_eq!(
        due_by_second,
        vec![
            vec![],
            vec![],
            refresh.clone(),
            vec![],
            refresh.clone(),
            vec![],
            refresh,
        ]
    );
}

#[test]
fn a_clock_that_jumps_ahead_makes_a_job_due_once_and_keeps_its_rhythm() {
    let mut timers = Timers::default();
    timers.every("refresh", 2, 100);

    assert_eq!(timers.due(111), vec!["refresh".to_string()]);
    assert_eq!(timers.next_at(), Some(112));
}

#[test]
fn next_at_is_the_earliest_job_and_none_without_jobs() {
    let mut timers = Timers::default();
    assert_eq!(timers.next_at(), None);

    timers.every("slow", 15, 100);
    timers.every("fast", 2, 100);

    assert_eq!(timers.next_at(), Some(102));
}

#[test]
fn registering_a_name_again_replaces_its_job() {
    let mut timers = Timers::default();
    timers.every("refresh", 2, 100);
    timers.every("refresh", 10, 100);

    assert_eq!(timers.due(102), Vec::<String>::new());
    assert_eq!(timers.next_at(), Some(110));
}

#[test]
fn a_job_of_zero_seconds_runs_each_second() {
    let mut timers = Timers::default();
    timers.every("eager", 0, 100);

    assert_eq!(timers.due(101), vec!["eager".to_string()]);
    assert_eq!(timers.next_at(), Some(102));
}
