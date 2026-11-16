use super::*;

const MEBIBYTE: u64 = 1024 * 1024;

fn run_id(number: usize) -> String {
    format!("20261116T{:06}Z-3f9a", number)
}

fn runs(count: usize, bytes: u64) -> Vec<StoredRun> {
    (1..=count)
        .map(|number| StoredRun {
            run_id: run_id(number),
            bytes,
        })
        .collect()
}

#[test]
fn the_hundred_and_first_run_drops_the_oldest() {
    assert_eq!(
        runs_to_drop(&Retention::default(), &runs(101, 1)),
        vec![run_id(1)]
    );
}

#[test]
fn a_hundred_runs_drop_nothing() {
    assert!(runs_to_drop(&Retention::default(), &runs(100, 1)).is_empty());
}

#[test]
fn runs_over_the_size_cap_drop_from_the_oldest() {
    let retention = Retention {
        keep_runs: 100,
        keep_bytes: 10 * MEBIBYTE,
    };

    assert_eq!(
        runs_to_drop(&retention, &runs(4, 4 * MEBIBYTE)),
        vec![run_id(1), run_id(2)]
    );
}

#[test]
fn the_newest_run_stays_even_when_it_alone_passes_the_cap() {
    let retention = Retention {
        keep_runs: 100,
        keep_bytes: MEBIBYTE,
    };

    assert_eq!(
        runs_to_drop(&retention, &runs(2, 5 * MEBIBYTE)),
        vec![run_id(1)]
    );
}

#[test]
fn runs_are_ordered_by_their_id_whatever_order_they_arrive_in() {
    let mut arrived = runs(101, 1);
    arrived.reverse();

    assert_eq!(
        runs_to_drop(&Retention::default(), &arrived),
        vec![run_id(1)]
    );
}
