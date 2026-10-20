use herdr_picker_domain::NewPane;
use serde_json::{Value, json};

use crate::wire::{ProtocolError, request_line};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneInfo {
    pub pane_id: String,
    pub workspace_id: String,
    pub tab_id: String,
    pub cwd: String,
    pub agent: Option<String>,
    pub status: String,
}

pub fn pane_request(request_id: &str, pane_id: &str) -> String {
    request_line(request_id, "pane.get", json!({ "pane_id": pane_id }))
}

pub fn pane_of(result: &Value) -> Result<PaneInfo, ProtocolError> {
    let pane = result
        .get("pane")
        .or_else(|| result.get("root_pane"))
        .ok_or_else(|| ProtocolError::Malformed("the result names no pane".to_string()))?;
    let text = |key: &str| pane.get(key).and_then(Value::as_str).map(str::to_string);
    Ok(PaneInfo {
        pane_id: text("pane_id")
            .ok_or_else(|| ProtocolError::Malformed("a pane has no pane_id".to_string()))?,
        workspace_id: text("workspace_id").unwrap_or_default(),
        tab_id: text("tab_id").unwrap_or_default(),
        cwd: text("cwd").unwrap_or_default(),
        agent: text("agent"),
        status: text("agent_status").unwrap_or_default(),
    })
}

pub fn screen_request(request_id: &str, pane_id: &str) -> String {
    let params = json!({ "pane_id": pane_id, "source": "visible" });
    request_line(request_id, "pane.read", params)
}

pub fn screen_of(result: &Value) -> Result<String, ProtocolError> {
    result
        .pointer("/read/text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ProtocolError::Malformed("the read has no text".to_string()))
}

pub fn type_request(request_id: &str, pane_id: &str, text: &str) -> String {
    let params = json!({ "pane_id": pane_id, "text": text });
    request_line(request_id, "pane.send_text", params)
}

pub fn enter_request(request_id: &str, pane_id: &str) -> String {
    let params = json!({ "pane_id": pane_id, "keys": ["enter"] });
    request_line(request_id, "pane.send_keys", params)
}

// herdr splits only right or down; a pane left or above is split there, then swapped.
pub fn open_request(request_id: &str, beside: &PaneInfo, pane: NewPane, folder: &str) -> String {
    let direction = match pane {
        NewPane::Left | NewPane::Right => "right",
        NewPane::Above | NewPane::Below => "down",
        NewPane::Tab => {
            let params =
                json!({ "workspace_id": beside.workspace_id, "cwd": folder, "focus": true });
            return request_line(request_id, "tab.create", params);
        }
    };
    let params = json!({
        "target_pane_id": beside.pane_id,
        "direction": direction,
        "cwd": folder,
        "focus": true,
    });
    request_line(request_id, "pane.split", params)
}

pub fn swap_request(request_id: &str, source: &str, target: &str) -> String {
    let params = json!({ "source_pane_id": source, "target_pane_id": target });
    request_line(request_id, "pane.swap", params)
}

pub fn notify_request(request_id: &str, body: &str) -> String {
    let params = json!({ "title": crate::PLUGIN_ID, "body": body });
    request_line(request_id, "notification.show", params)
}

#[cfg(test)]
mod tests;
