use serde_json::Value;

use super::{ForegroundProcess, ProgramStatus, RexBlock, RexEvent};

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_string)
}

pub(super) fn session_event(event: &Value) -> Option<RexEvent> {
    if event.get("type")?.as_str()? != "session_event" {
        return None;
    }
    let payload = event.get("payload")?;
    let rex_session = text(payload, "session_id")?;
    let inner = payload.get("event")?;
    let details = inner.get("payload")?;
    let block = |rex_session: String| {
        Some(RexBlock {
            rex_session,
            block: text(details, "block_id")?,
        })
    };
    match inner.get("type")?.as_str()? {
        "active_window_changed" => Some(RexEvent::TabChanged {
            rex_session,
            tab: text(details, "window_id")?,
        }),
        "session_view_changed" => Some(RexEvent::ViewChanged { rex_session }),
        "block_closed" => Some(RexEvent::BlockClosed {
            at: block(rex_session)?,
        }),
        "block_event" => block_event(block(rex_session)?, details),
        _ => None,
    }
}

fn block_event(at: RexBlock, details: &Value) -> Option<RexEvent> {
    let reported = details.get("payload")?;
    match details.get("name")?.as_str()? {
        "program_status_changed" => Some(RexEvent::ProgramStatusChanged {
            at,
            status: program_status(reported.get("record")?)?,
        }),
        "program_status_removed" => Some(RexEvent::ProgramStatusRemoved { at }),
        "process_changed" => Some(RexEvent::ProcessChanged {
            at,
            foreground: reported.get("foreground").and_then(foreground_process),
        }),
        "child_exited" => Some(RexEvent::ChildExited {
            at,
            exit_code: reported.get("exit_code").and_then(Value::as_i64),
        }),
        _ => None,
    }
}

fn pid(value: &Value) -> Option<u32> {
    value.get("pid")?.as_u64()?.try_into().ok()
}

fn program_status(record: &Value) -> Option<ProgramStatus> {
    Some(ProgramStatus {
        state: text(record, "state")?,
        kind: text(record, "kind"),
        program: text(record, "app")?,
        message: text(record, "msg"),
        pid: record.get("owner").and_then(pid),
    })
}

fn foreground_process(foreground: &Value) -> Option<ForegroundProcess> {
    Some(ForegroundProcess {
        pid: pid(foreground)?,
        name: text(foreground, "name")?,
    })
}
