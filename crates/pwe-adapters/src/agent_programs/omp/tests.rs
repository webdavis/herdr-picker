use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use pwe_application::{AgentProgram, OpenFile, OpenFiles};
use pwe_domain::AgentState;

use crate::cli_command::{RecordedCli, spawning};

use super::*;

const MAIN: &str = "44444444-4444-7444-8444-444444444444";
const ADVISOR: &str = "55555555-5555-7555-8555-555555555555";
const LEASE: &str = r#"{"held":["44444444-4444-7444-8444-444444444444","55555555-5555-7555-8555-555555555555"],"unknown":[]}"#;
const SESSION_FILE: &str = concat!(
    r#"{"type":"session","version":3,"id":"44444444-4444-7444-8444-444444444444","timestamp":"t","cwd":"/work/project"}"#,
    "\n",
    r#"{"type":"title_change","id":"a1","source":"auto","title":"Greeting check"}"#,
    "\n",
    r#"{"type":"message","id":"a2","message":{"role":"assistant","stopReason":"stop"}}"#,
    "\n"
);

struct HeldBy(u32);

impl OpenFiles for HeldBy {
    fn opened_by(&self, command: &str) -> Vec<OpenFile> {
        assert_eq!(command, "omp");
        vec![OpenFile {
            pid: self.0,
            path: PathBuf::from(format!("/somewhere/run/session-owners/{MAIN}.lock")),
        }]
    }
}

fn omp_folder(fake: &RecordedCli) -> PathBuf {
    let folder = fake.folder().join("dot-omp");
    let owners = folder.join("run/session-owners");
    let sessions = folder.join("agent/sessions/--work-project--");
    fs::create_dir_all(&owners).expect("owners folder");
    fs::create_dir_all(sessions.join(MAIN)).expect("sessions folder");
    for id in [MAIN, ADVISOR] {
        fs::write(owners.join(format!("{id}.lock")), "").expect("a lock file");
    }
    fs::write(
        sessions.join(format!("2026-01-01T00-00-00-000Z_{MAIN}.jsonl")),
        SESSION_FILE,
    )
    .expect("a session file");
    fs::write(sessions.join(MAIN).join("__advisor.jsonl"), "").expect("a subsession file");
    folder
}

fn path_of(folder: &Path) -> OsString {
    std::env::join_paths([folder]).expect("a PATH")
}

#[test]
fn a_held_lease_with_a_session_file_is_one_session() {
    let _guard = spawning();
    let omp = RecordedCli::printing("omp", LEASE);
    let folder = omp_folder(&omp);

    let sessions = OmpSessions::on(Some(&path_of(omp.folder())), &folder, HeldBy(102))
        .sessions()
        .expect("omp answered");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, MAIN);
    assert_eq!(sessions[0].name.as_deref(), Some("Greeting check"));
    assert_eq!(sessions[0].pid, Some(102));
    assert_eq!(sessions[0].state, AgentState::Idle);
    assert!(sessions[0].state_since.is_some());
    let asked = omp.calls().remove(0);
    assert_eq!(asked[..4], ["session", "lease", "--json", "--"]);
    let mut ids = asked[4..].to_vec();
    ids.sort();
    assert_eq!(ids, [MAIN, ADVISOR]);
}

#[test]
fn a_missing_omp_is_skipped() {
    let _guard = spawning();
    let elsewhere = RecordedCli::printing("not-omp", "");
    let folder = omp_folder(&elsewhere);

    let sessions =
        OmpSessions::on(Some(&path_of(elsewhere.folder())), &folder, HeldBy(1)).sessions();

    assert_eq!(sessions, Ok(Vec::new()));
}
