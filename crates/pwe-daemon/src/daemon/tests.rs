use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::process;
use std::thread::{self, JoinHandle};

use pwe_adapters::{DaemonAskError, ask_daemon};
use pwe_protocol::{DaemonRequest, DaemonStatus, Listens, RexLink, parse_daemon_status};

const STARTED_AT: u64 = 1_700_000_000;

struct TempFolder(PathBuf);

impl TempFolder {
    fn new(name: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("pwed-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&folder);
        Self(folder)
    }

    fn socket(&self) -> PathBuf {
        self.0.join("pwe/daemon.sock")
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn serving(socket: &Path) -> JoinHandle<io::Result<()>> {
    let daemon = Daemon::claim(socket, STARTED_AT)
        .unwrap_or_else(|refusal| panic!("the daemon did not start: {refusal}"));
    thread::spawn(move || daemon.serve())
}

fn stopped(socket: &Path, serving: JoinHandle<io::Result<()>>) {
    ask_daemon(socket, DaemonRequest::Stop)
        .unwrap_or_else(|refusal| panic!("the daemon did not stop: {refusal}"));
    match serving.join() {
        Ok(served) => served.unwrap_or_else(|failure| panic!("serving failed: {failure}")),
        Err(_) => panic!("the daemon thread panicked"),
    }
}

fn status_of(socket: &Path) -> DaemonStatus {
    ask_daemon(socket, DaemonRequest::Status)
        .map_err(|refusal| refusal.to_string())
        .and_then(|result| parse_daemon_status(&result).map_err(|refusal| refusal.message))
        .unwrap_or_else(|refusal| panic!("no status: {refusal}"))
}

fn mode_of(path: &Path) -> u32 {
    fs::metadata(path)
        .unwrap_or_else(|failure| panic!("{}: {failure}", path.display()))
        .permissions()
        .mode()
        & 0o777
}

#[test]
fn status_answers_with_this_machine_only() {
    let folder = TempFolder::new("status");
    let serving = serving(&folder.socket());

    let status = status_of(&folder.socket());
    stopped(&folder.socket(), serving);

    assert_eq!(
        status,
        DaemonStatus {
            running: true,
            pid: process::id(),
            socket: folder.socket().to_string_lossy().into_owned(),
            listens: Listens::ThisMachineOnly,
            started_at: STARTED_AT,
            rex: RexLink::Absent,
        }
    );
}

#[test]
fn a_second_daemon_refuses_to_start() {
    let folder = TempFolder::new("second");
    let serving = serving(&folder.socket());

    let second = Daemon::claim(&folder.socket(), STARTED_AT);
    let still_running = status_of(&folder.socket()).running;
    stopped(&folder.socket(), serving);

    assert!(
        matches!(second, Err(DaemonStartError::AlreadyRunning(_))),
        "{:?}",
        second.err()
    );
    assert!(still_running);
}

#[test]
fn a_stale_socket_file_is_replaced() {
    let folder = TempFolder::new("stale");
    fs::create_dir_all(folder.0.join("pwe")).unwrap_or_else(|failure| panic!("{failure}"));
    drop(UnixListener::bind(folder.socket()).unwrap_or_else(|failure| panic!("{failure}")));
    assert!(folder.socket().exists());

    let serving = serving(&folder.socket());
    let status = status_of(&folder.socket());
    stopped(&folder.socket(), serving);

    assert!(status.running);
}

#[test]
fn the_socket_is_readable_by_its_owner_only() {
    let folder = TempFolder::new("private");
    let serving = serving(&folder.socket());

    let socket_mode = mode_of(&folder.socket());
    let folder_mode = mode_of(&folder.0.join("pwe"));
    stopped(&folder.socket(), serving);

    assert_eq!((folder_mode, socket_mode), (0o700, 0o600));
}

#[test]
fn stop_ends_the_daemon_and_removes_its_socket() {
    let folder = TempFolder::new("stop");
    let serving = serving(&folder.socket());

    stopped(&folder.socket(), serving);

    assert!(!folder.socket().exists());
    assert!(matches!(
        ask_daemon(&folder.socket(), DaemonRequest::Status),
        Err(DaemonAskError::NotRunning)
    ));
}

#[test]
fn the_loop_wakes_for_a_job_without_any_request() {
    let folder = TempFolder::new("clock");
    let mut daemon = Daemon::claim(&folder.socket(), STARTED_AT)
        .unwrap_or_else(|refusal| panic!("the daemon did not start: {refusal}"));
    let (ran, told) = std::sync::mpsc::channel();
    daemon.every("ping", 1, move |_| {
        let _ = ran.send(());
    });
    let serving = thread::spawn(move || daemon.serve());

    let woke = told.recv_timeout(std::time::Duration::from_secs(3));
    stopped(&folder.socket(), serving);

    assert!(woke.is_ok(), "the job never ran");
}

fn fake_rex(folder: &TempFolder) -> PathBuf {
    let binary = folder.0.join("rex");
    let event = r#"{"type":"session_event","payload":{"session_id":"session:00000000-0000-7000-8000-00000000a001","event":{"type":"session_view_changed","payload":{"revision":1}}}}"#;
    fs::write(
        &binary,
        format!("#!/bin/sh\nprintf '%s\\n' '{event}'\nexec sleep 30\n"),
    )
    .unwrap_or_else(|failure| panic!("{failure}"));
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700))
        .unwrap_or_else(|failure| panic!("{failure}"));
    binary
}

#[test]
fn status_says_rex_is_connected_once_its_events_flow() {
    let folder = TempFolder::new("rex");
    fs::create_dir_all(&folder.0).unwrap_or_else(|failure| panic!("{failure}"));
    let mut daemon = Daemon::claim(&folder.socket(), STARTED_AT)
        .unwrap_or_else(|refusal| panic!("the daemon did not start: {refusal}"));
    daemon.follow_rex(Some(fake_rex(&folder)));
    let serving = thread::spawn(move || daemon.serve());

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut link = status_of(&folder.socket()).rex;
    while link != RexLink::Connected && std::time::Instant::now() < deadline {
        thread::sleep(std::time::Duration::from_millis(50));
        link = status_of(&folder.socket()).rex;
    }
    stopped(&folder.socket(), serving);

    assert_eq!(link, RexLink::Connected);
}

#[test]
fn without_a_rex_program_status_says_rex_is_absent() {
    let folder = TempFolder::new("absent");
    let mut daemon = Daemon::claim(&folder.socket(), STARTED_AT)
        .unwrap_or_else(|refusal| panic!("the daemon did not start: {refusal}"));
    daemon.follow_rex(None);
    let serving = thread::spawn(move || daemon.serve());

    let link = status_of(&folder.socket()).rex;
    stopped(&folder.socket(), serving);

    assert_eq!(link, RexLink::Absent);
}
