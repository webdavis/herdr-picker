use std::fs;
use std::path::PathBuf;

use super::*;

struct TempState(PathBuf);

impl TempState {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hp-places-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Self(dir)
    }

    fn path(&self) -> PathBuf {
        self.0.join("places")
    }
}

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn place(pane_id: &str, cwd: &str, at: u64) -> Place {
    Place {
        at,
        workspace_id: "w1".to_string(),
        tab_id: "w1:t1".to_string(),
        pane_id: pane_id.to_string(),
        cwd: cwd.to_string(),
    }
}

#[test]
fn recent_places_come_back_newest_first_from_the_lines_written() {
    let state = TempState::new("order");

    assert_eq!(recent_places(&state.path()), Vec::new());
    record_place(&state.path(), place("w1:p1", "/a", 10), 5);
    record_place(&state.path(), place("w1:p2", "/b b", 20), 5);

    assert_eq!(
        fs::read_to_string(state.path()).expect("the log"),
        "10\tw1\tw1:t1\tw1:p1\t/a\n20\tw1\tw1:t1\tw1:p2\t/b b\n"
    );
    assert_eq!(
        recent_places(&state.path()),
        vec![place("w1:p2", "/b b", 20), place("w1:p1", "/a", 10)]
    );
}

#[test]
fn the_log_keeps_only_the_newest_places_up_to_its_bound() {
    let state = TempState::new("bound");

    for at in 1..=4 {
        record_place(&state.path(), place(&format!("w1:p{at}"), "/a", at), 3);
    }

    assert_eq!(
        recent_places(&state.path())
            .into_iter()
            .map(|place| place.at)
            .collect::<Vec<_>>(),
        vec![4, 3, 2]
    );
}

#[test]
fn a_folder_that_would_break_a_line_is_not_logged_and_a_broken_line_is_skipped() {
    let state = TempState::new("odd");
    fs::create_dir_all(&state.0).expect("the state folder");
    fs::write(state.path(), "garbage\n5\tw1\tw1:t1\tw1:p1\t/a\n").expect("a log");

    record_place(&state.path(), place("w1:p2", "/new\nline", 6), 5);

    assert_eq!(recent_places(&state.path()), vec![place("w1:p1", "/a", 5)]);
}
