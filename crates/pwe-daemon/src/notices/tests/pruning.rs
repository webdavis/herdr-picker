use pwe_adapters::save_inbox_row;
use pwe_domain::Notice;

use super::*;

const DAY: u64 = 86_400;

fn read_row(id: &str, read_at: u64) -> Notice {
    Notice {
        id: id.to_string(),
        key: id.to_string(),
        source: "tests".to_string(),
        kind: "run".to_string(),
        level: Level::Finished,
        text: "Tests passed".to_string(),
        pane: None,
        tab_label: None,
        raised_at: read_at,
        updated_at: read_at,
        resolved_at: None,
        read_at: Some(read_at),
    }
}

fn ids(inbox: &std::path::Path) -> Vec<String> {
    read_inbox(inbox)
        .unwrap_or_default()
        .into_iter()
        .map(|row| row.id)
        .collect()
}

#[test]
fn the_inbox_is_pruned_at_start_and_once_a_day() {
    let folder = std::env::temp_dir().join(format!("pwed-prune-{}", process::id()));
    let _ = fs::remove_dir_all(&folder);
    let inbox = folder.join("inbox.jsonl");
    let save =
        |row: Notice| save_inbox_row(&inbox, &row).unwrap_or_else(|failure| panic!("{failure}"));
    save(read_row("old", NOW - 40 * DAY));
    save(read_row("recent", NOW - 20 * DAY));
    let (parts, _calls) = fake_parts(inbox.clone(), false);

    let mut desk = NoticeDesk::new(parts, NOW);
    let after_start = ids(&inbox);
    desk.run_due(NOW + DAY - 1);
    let before_a_day = ids(&inbox);
    desk.run_due(NOW + 11 * DAY);
    let _ = fs::remove_dir_all(&folder);

    assert_eq!(after_start, vec!["recent"]);
    assert_eq!(before_a_day, vec!["recent"]);
    assert_eq!(ids(&inbox), Vec::<String>::new());
}
