use super::*;
use pwe_protocol::{DaemonErrorKind, Listens, RexLink, parse_daemon_reply};
use std::sync::{Arc, Mutex};

fn core() -> DaemonCore {
    DaemonCore::new(DaemonStatus {
        running: true,
        pid: 7,
        socket: "/run/pwe/daemon.sock".to_string(),
        listens: Listens::ThisMachineOnly,
        started_at: 1,
        rex: RexLink::Absent,
    })
}

#[test]
fn an_unknown_request_is_refused_and_the_daemon_keeps_serving() {
    let answer = core().answer(r#"{"pwe-daemon":1,"request":"dance"}"#);

    let refusal = parse_daemon_reply(&answer.reply_line).err();
    assert_eq!(
        refusal.map(|refusal| refusal.kind),
        Some(DaemonErrorKind::UnknownRequest)
    );
    assert!(answer.keeps_serving);
}

#[test]
fn stop_is_answered_and_ends_serving() {
    let answer = core().answer(r#"{"pwe-daemon":1,"request":"stop"}"#);

    assert_eq!(
        parse_daemon_reply(&answer.reply_line),
        Ok(json!({ "stopping": true }))
    );
    assert!(!answer.keeps_serving);
}

fn recording_job() -> (Arc<Mutex<Vec<u64>>>, Job) {
    let runs = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&runs);
    let job: Job = Box::new(move |now| {
        if let Ok(mut runs) = recorded.lock() {
            runs.push(now);
        }
    });
    (runs, job)
}

fn runs_of(runs: &Arc<Mutex<Vec<u64>>>) -> Vec<u64> {
    runs.lock().map(|runs| runs.clone()).unwrap_or_default()
}

#[test]
fn a_job_runs_when_the_fake_clock_passes_its_time() {
    let mut core = core();
    let (runs, job) = recording_job();
    core.every("refresh", 15, 100, job);

    core.run_due(114);
    let before = runs_of(&runs);
    core.run_due(116);

    assert_eq!(before, Vec::<u64>::new());
    assert_eq!(runs_of(&runs), vec![116]);
    assert_eq!(core.next_at(), Some(130));
}

#[test]
fn without_jobs_there_is_no_next_time() {
    assert_eq!(core().next_at(), None);
}
