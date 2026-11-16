use super::*;
use std::os::unix::fs::PermissionsExt;

use pwe_domain::{Pane, ProgramName};

fn recorded() -> RecordedPane {
    RecordedPane {
        program: ProgramName::ClaudeCode,
        session_id: "00000000-0000-4000-8000-000000000042".to_string(),
        folder: Some("/work/project".to_string()),
        pane: Pane {
            rex_session: "session:00000000-0000-7000-8000-0000000000a1".to_string(),
            tab: "window:00000000-0000-7000-8000-0000000000c1".to_string(),
            block: "block:00000000-0000-7000-8000-0000000000b1".to_string(),
        },
    }
}

#[test]
fn saved_panes_are_read_back_from_a_file_only_its_owner_reads() {
    let folder = std::env::temp_dir().join(format!("pwe-agent-panes-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    let file = folder.join("pwe/agent-panes.json");

    let saved = save_agent_panes(&file, &[recorded()]);
    let read = read_agent_panes(&file);
    let mode = fs::metadata(&file).map(|found| found.permissions().mode() & 0o777);
    let _ = fs::remove_dir_all(&folder);

    assert!(saved.is_ok(), "{saved:?}");
    assert_eq!(read.ok(), Some(vec![recorded()]));
    assert_eq!(mode.ok(), Some(0o600));
}

#[test]
fn a_missing_file_holds_no_panes() {
    assert_eq!(
        read_agent_panes(Path::new("/nonexistent/pwe/agent-panes.json")).ok(),
        Some(Vec::new())
    );
}

#[test]
fn a_record_file_others_may_write_is_refused() {
    let folder = std::env::temp_dir().join(format!("pwe-agent-panes-open-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    let file = folder.join("agent-panes.json");
    let saved = save_agent_panes(&file, &[recorded()]);
    let opened = fs::set_permissions(&file, fs::Permissions::from_mode(0o666));

    let read = read_agent_panes(&file);
    let _ = fs::remove_dir_all(&folder);

    assert!(saved.is_ok() && opened.is_ok());
    assert!(read.is_err(), "{read:?}");
}
