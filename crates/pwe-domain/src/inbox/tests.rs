use super::*;
use crate::notice::Level;

const DAY: u64 = 86_400;

fn row(id: &str, updated_at: u64, read_at: Option<u64>) -> Notice {
    Notice {
        id: id.to_string(),
        key: id.to_string(),
        source: "tests".to_string(),
        kind: "build".to_string(),
        level: Level::Finished,
        text: "done".to_string(),
        pane: None,
        tab_label: None,
        raised_at: updated_at,
        updated_at,
        resolved_at: None,
        read_at,
    }
}

fn ids(rows: &[Notice]) -> Vec<&str> {
    rows.iter().map(|row| row.id.as_str()).collect()
}

#[test]
fn read_rows_older_than_keep_days_are_pruned_and_unread_rows_stay() {
    let now = 100 * DAY;
    let rows = vec![
        row("old-read", now - 40 * DAY, Some(now - 40 * DAY)),
        row("old-unread", now - 40 * DAY, None),
        row("new-read", now - 2 * DAY, Some(now - DAY)),
    ];

    let kept = prune_inbox(rows, now, &InboxLimits::default());

    assert_eq!(ids(&kept), vec!["old-unread", "new-read"]);
}

#[test]
fn a_row_read_recently_counts_from_when_it_was_read() {
    let now = 100 * DAY;
    let rows = vec![row("read-yesterday", now - 40 * DAY, Some(now - DAY))];

    assert_eq!(prune_inbox(rows, now, &InboxLimits::default()).len(), 1);
}

#[test]
fn past_most_rows_the_oldest_read_rows_go_first_and_unread_rows_stay() {
    let limits = InboxLimits {
        keep_days: 30,
        most_rows: 2,
    };
    let rows = vec![
        row("unread", 10, None),
        row("older-read", 20, Some(20)),
        row("newer-read", 30, Some(30)),
        row("also-unread", 40, None),
    ];

    assert_eq!(
        ids(&prune_inbox(rows.clone(), 50, &limits)),
        vec!["unread", "also-unread"]
    );
    let one_more = InboxLimits {
        most_rows: 3,
        ..limits
    };
    assert_eq!(
        ids(&prune_inbox(rows, 50, &one_more)),
        vec!["unread", "newer-read", "also-unread"]
    );
}
