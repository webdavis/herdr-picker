use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};

use pwe_adapters::read_inbox;
use pwe_domain::{Level, NoticeInput, NoticeSettings, OwnerPlace, Raised, RexFocus};

use super::*;
use fakes::{
    Calls, FakeNotifier, FakePlaces, FakeScreen, FakeWhereabouts, SESSION, in_front_of, pane,
};

mod fakes;

const NOW: u64 = 1_790_000_100;
const OWNER_PANE: &str = "block:00000000-0000-4000-8000-0000000000b1";
const OTHER_PANE: &str = "block:00000000-0000-4000-8000-0000000000b2";

struct Bench {
    desk: NoticeDesk,
    screen: Calls,
    macos: Calls,
    asked: Calls,
    log: Calls,
    place: Arc<Mutex<OwnerPlace>>,
    inbox: PathBuf,
}

impl Bench {
    fn new(name: &str, screen_fails: bool) -> Self {
        let folder = std::env::temp_dir().join(format!("pwed-notices-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&folder);
        let (screen, macos, asked, log) = Default::default();
        let place = Arc::new(Mutex::new(in_front_of(SESSION)));
        let inbox = folder.join("inbox.jsonl");
        let parts = NoticeParts {
            settings: NoticeSettings::default(),
            screen: Box::new(FakeScreen {
                calls: Calls::clone(&screen),
                fails: screen_fails,
            }),
            notifier: Box::new(FakeNotifier(Calls::clone(&macos))),
            whereabouts: Box::new(FakeWhereabouts {
                calls: Calls::clone(&asked),
                place: Arc::clone(&place),
            }),
            places: Box::new(FakePlaces {
                focused: Arc::new(Mutex::new(pane(OWNER_PANE))),
            }),
            inbox: inbox.clone(),
            log: Box::new(Calls::clone(&log)),
        };
        Self {
            desk: NoticeDesk::new(parts, NOW),
            screen,
            macos,
            asked,
            log,
            place,
            inbox,
        }
    }

    fn raise(&mut self, key: &str, level: Level, now: u64) {
        let raised = Raised {
            key: key.to_string(),
            source: "tests".to_string(),
            kind: "run".to_string(),
            level,
            text: "Tests passed".to_string(),
            pane: Some(pane(OTHER_PANE)),
            tab_label: Some("api".to_string()),
        };
        self.desk.apply(NoticeInput::Raise(raised), now);
    }

    fn owner_looks_at_rex(&mut self, now: u64) {
        self.desk.apply(
            NoticeInput::OwnerMoved(RexFocus::Front(pane(OWNER_PANE))),
            now,
        );
    }

    fn run_clock(&mut self, from: u64, to: u64) {
        for now in from..=to {
            self.desk.run_due(now);
        }
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        if let Some(folder) = self.inbox.parent() {
            let _ = fs::remove_dir_all(folder);
        }
    }
}

#[test]
fn the_session_is_asked_for_only_while_a_notice_waits() {
    let mut bench = Bench::new("asking", false);
    bench.run_clock(NOW, NOW + 10);
    let before_any_notice = bench.asked.taken();

    bench.raise("agent-1", Level::NeedsYou, NOW + 10);
    bench.run_clock(NOW + 11, NOW + 14);
    let while_waiting = bench.asked.taken();
    bench.desk.apply(
        NoticeInput::Resolve {
            key: "agent-1".to_string(),
        },
        NOW + 14,
    );
    bench.run_clock(NOW + 15, NOW + 30);

    assert_eq!(before_any_notice, Vec::<String>::new());
    assert_eq!(while_waiting, vec!["where", "where"]);
    assert_eq!(bench.asked.taken(), Vec::<String>::new());
}

#[test]
fn a_finished_notice_shows_in_the_session_in_view_and_closes_on_the_daemon_clock() {
    let mut bench = Bench::new("expiry", false);
    bench.owner_looks_at_rex(NOW);

    bench.raise("tests", Level::Finished, NOW);
    let shown = bench.screen.taken();
    bench.run_clock(NOW + 1, NOW + 5);
    let before_its_seconds = bench.screen.taken();
    bench.run_clock(NOW + 6, NOW + 6);

    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].starts_with(&format!(
            "show float-2 in {SESSION} slot 0 until {}",
            NOW + 6
        )),
        "{shown:?}"
    );
    assert_eq!(before_its_seconds, Vec::<String>::new());
    assert_eq!(bench.screen.taken(), vec!["close float-2"]);
}

#[test]
fn with_rex_in_the_background_a_notice_goes_to_macos() {
    let mut bench = Bench::new("macos", false);
    bench
        .desk
        .apply(NoticeInput::OwnerMoved(RexFocus::Background), NOW);

    bench.raise("tests", Level::Finished, NOW);

    assert_eq!(
        bench.macos.taken(),
        vec!["macos finished: tests: Tests passed, in tab api"]
    );
    assert_eq!(bench.screen.taken(), Vec::<String>::new());
}

#[test]
fn a_failed_screen_is_logged_and_the_row_is_still_saved() {
    let mut bench = Bench::new("failing", true);
    bench.owner_looks_at_rex(NOW);

    bench.raise("tests", Level::Finished, NOW);
    bench.run_clock(NOW + 1, NOW + 6);

    let rows = read_inbox(&bench.inbox).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text, "Tests passed");
    let log = bench.log.taken();
    assert_eq!(log.len(), 2, "{log:?}");
    assert!(
        log.iter().all(|line| line.contains("rex is gone")),
        "{log:?}"
    );
}

#[test]
fn the_answer_to_the_session_ask_becomes_the_boards_focus() {
    let mut bench = Bench::new("focus", false);
    bench.owner_looks_at_rex(NOW);
    bench.raise("agent-1", Level::NeedsYou, NOW);
    if let Ok(mut place) = bench.place.lock() {
        *place = OwnerPlace::Closed;
    }

    bench.run_clock(NOW + 1, NOW + 2);
    bench.raise("tests", Level::Finished, NOW + 2);

    assert_eq!(
        bench.macos.taken(),
        vec!["macos finished: tests: Tests passed, in tab api"]
    );
}

#[test]
fn the_daemon_clock_wakes_each_second_for_notices() {
    let bench = Bench::new("clock", false);

    assert_eq!(bench.desk.next_at(), Some(NOW + 1));
}
