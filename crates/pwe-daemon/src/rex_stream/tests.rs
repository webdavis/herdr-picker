use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{self, Command};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use pwe_protocol::RexEvent;

const TAB_CHANGED: &str = r#"{"type":"session_event","payload":{"session_id":"session:00000000-0000-7000-8000-00000000a001","event":{"type":"active_window_changed","payload":{"window_id":"window:00000000-0000-7000-8000-00000000b001"}}}}"#;
const PATIENCE: Duration = Duration::from_secs(30);

struct FakeRex(PathBuf);

impl FakeRex {
    fn running(name: &str, body: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("pwe-rexs-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap_or_else(|failure| panic!("{failure}"));
        let script = format!(
            "#!/bin/sh\ncd '{}'\nprintf '%s\\n' \"$*\" >> calls\n{body}\n",
            folder.display()
        );
        let binary = folder.join("rex");
        fs::write(&binary, script).unwrap_or_else(|failure| panic!("{failure}"));
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700))
            .unwrap_or_else(|failure| panic!("{failure}"));
        Self(folder)
    }

    fn stream(&self) -> RexEventStream {
        RexEventStream::at(self.0.join("rex"))
    }

    fn file(&self, name: &str) -> String {
        fs::read_to_string(self.0.join(name)).unwrap_or_default()
    }
}

impl Drop for FakeRex {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn news_until(arrived: &Receiver<Arrival>, enough: impl Fn(&[RexNews]) -> bool) -> Vec<RexNews> {
    let deadline = Instant::now() + PATIENCE;
    let mut heard = Vec::new();
    while !enough(&heard) && Instant::now() < deadline {
        if let Ok(Arrival::Rex(news)) = arrived.recv_timeout(Duration::from_millis(200)) {
            heard.push(news);
        }
    }
    heard
}

fn tab_changes(heard: &[RexNews]) -> usize {
    heard
        .iter()
        .filter(|news| matches!(news, RexNews::Event(RexEvent::TabChanged { .. })))
        .count()
}

fn is_running(pid: &str) -> bool {
    Command::new("kill")
        .args(["-0", pid.trim()])
        .status()
        .is_ok_and(|status| status.success())
}

fn wait_for_file(rex: &FakeRex, name: &str) -> String {
    let deadline = Instant::now() + PATIENCE;
    while rex.file(name).is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    rex.file(name)
}

#[test]
fn the_stream_opens_again_after_rex_goes_away() {
    let rex = FakeRex::running("again", &format!("printf '%s\\n' '{TAB_CHANGED}'"));
    let (arrivals, arrived) = mpsc::sync_channel(64);
    let follower = RexFollower::start(rex.stream(), arrivals);

    let heard = news_until(&arrived, |heard| tab_changes(heard) >= 2);
    follower.stop();

    assert_eq!(tab_changes(&heard), 2, "{heard:?}");
    let first_tab = heard
        .iter()
        .position(|news| matches!(news, RexNews::Event(_)));
    let gone = heard
        .iter()
        .position(|news| *news == RexNews::Linked(RexLink::Reconnecting));
    assert!(first_tab < gone, "{heard:?}");
    assert_eq!(heard[0], RexNews::Linked(RexLink::Connected));
    assert!(
        rex.file("calls")
            .lines()
            .all(|call| call == "--autostart=false events --all --json"),
        "{}",
        rex.file("calls")
    );
}

#[test]
fn an_unreadable_line_is_skipped_and_the_next_one_read() {
    let rex = FakeRex::running(
        "unreadable",
        &format!("printf 'not an event\\n%s\\n' '{TAB_CHANGED}'\nexec sleep 30"),
    );
    let (arrivals, arrived) = mpsc::sync_channel(64);
    let follower = RexFollower::start(rex.stream(), arrivals);

    let heard = news_until(&arrived, |heard| tab_changes(heard) >= 1);
    follower.stop();

    assert_eq!(tab_changes(&heard), 1, "{heard:?}");
}

#[test]
fn stopping_ends_the_rex_child() {
    let rex = FakeRex::running(
        "stopping",
        &format!("printf '%s\\n' '{TAB_CHANGED}'\necho $$ > pid\nexec sleep 30"),
    );
    let (arrivals, arrived) = mpsc::sync_channel(64);
    let follower = RexFollower::start(rex.stream(), arrivals);
    news_until(&arrived, |heard| tab_changes(heard) >= 1);
    let pid = wait_for_file(&rex, "pid");
    assert!(is_running(&pid), "the fake rex never ran");

    follower.stop();

    assert!(!is_running(&pid), "process {pid} still runs");
}

#[test]
fn the_follower_stops_while_rex_writes_faster_than_the_daemon_reads() {
    let rex = FakeRex::running(
        "full",
        &format!("while :; do printf '%s\\n' '{TAB_CHANGED}'; done"),
    );
    let (arrivals, arrived) = mpsc::sync_channel(1);
    let follower = RexFollower::start(rex.stream(), arrivals);
    news_until(&arrived, |heard| !heard.is_empty());

    drop(arrived);
    follower.stop();
}
