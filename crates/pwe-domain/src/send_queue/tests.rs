use super::*;

fn queued(pane_id: &str, text: &str) -> Queued {
    Queued {
        pane_id: pane_id.to_string(),
        text: text.to_string(),
    }
}

#[test]
fn the_panes_oldest_prompt_goes_and_the_rest_stay_in_order() {
    let queue = vec![
        queued("p1", "first"),
        queued("p2", "other"),
        queued("p1", "second"),
    ];

    assert_eq!(
        take_next(queue, "p1"),
        (
            Some(queued("p1", "first")),
            vec![queued("p2", "other"), queued("p1", "second")]
        )
    );
}

#[test]
fn a_pane_with_nothing_waiting_takes_nothing() {
    let queue = vec![queued("p2", "other")];

    assert_eq!(take_next(queue, "p1"), (None, vec![queued("p2", "other")]));
}
