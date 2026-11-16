mod options;

use std::path::Path;
use std::process;

use pwe_adapters::{DaemonAskError, ask_daemon_line, inbox_file, read_inbox, save_inbox_row};
use pwe_daemon::seconds_since_1970;
use pwe_domain::{Notice, fit_label};
use pwe_protocol::{NoticeRequest, NotifyRequest, notice_request_line};
use serde_json::{Value, json};

use crate::command::{CommandError, failed, state_dir};
pub(crate) use options::request;

pub(crate) const DAEMON_STOPPED: &str = "the pwe daemon is not running, so the notice was saved to the inbox only; run `pwe daemon start` to show notices";
const DAEMON_STOPPED_EXIT: u8 = 3;

#[derive(Debug)]
pub(crate) struct Sent {
    pub(crate) printed: String,
    pub(crate) daemon_stopped: bool,
}

pub(crate) fn run(socket: &Path, options: &[&str]) -> Result<(), CommandError> {
    let now = seconds_since_1970();
    let request = request(options, now, process::id())?;
    let sent = send(socket, &inbox_file(&state_dir()), &request, now)?;
    println!("{}", sent.printed);
    if sent.daemon_stopped {
        eprintln!("pwe: {DAEMON_STOPPED}");
        return Err(CommandError::Exit(DAEMON_STOPPED_EXIT));
    }
    Ok(())
}

pub(crate) fn send(
    socket: &Path,
    inbox: &Path,
    request: &NoticeRequest,
    now: u64,
) -> Result<Sent, CommandError> {
    let (result, daemon_stopped) = match ask_daemon_line(socket, &notice_request_line(request)) {
        Ok(result) => (result, false),
        Err(DaemonAskError::NotRunning) => (save_alone(inbox, request, now)?, true),
        Err(refusal) => return Err(failed(refusal)),
    };
    Ok(Sent {
        printed: result.to_string(),
        daemon_stopped,
    })
}

fn save_alone(inbox: &Path, request: &NoticeRequest, now: u64) -> Result<Value, CommandError> {
    let unsaved = |failure| {
        failed(format!(
            "could not save to the inbox {}: {failure}",
            inbox.display()
        ))
    };
    match request {
        NoticeRequest::Notify(notify) => {
            let row = row_of(notify, now);
            save_inbox_row(inbox, &row).map_err(unsaved)?;
            Ok(json!({ "id": row.id }))
        }
        NoticeRequest::Resolve { key } => {
            let mut open: Vec<Notice> = read_inbox(inbox)
                .map_err(unsaved)?
                .into_iter()
                .filter(|row| &row.key == key && row.resolved_at.is_none())
                .collect();
            for row in &mut open {
                row.resolved_at = Some(now);
                save_inbox_row(inbox, row).map_err(unsaved)?;
            }
            Ok(json!({ "resolved": !open.is_empty() }))
        }
        _ => Err(CommandError::Usage),
    }
}

fn row_of(notify: &NotifyRequest, now: u64) -> Notice {
    Notice {
        id: format!("{now}-cli-{}", process::id()),
        key: fit_label(&notify.key),
        source: fit_label(&notify.source),
        kind: fit_label(&notify.kind),
        level: notify.level,
        text: fit_label(&notify.text),
        pane: None,
        tab_label: None,
        raised_at: now,
        updated_at: now,
        resolved_at: None,
        read_at: None,
    }
}

#[cfg(test)]
mod tests;
