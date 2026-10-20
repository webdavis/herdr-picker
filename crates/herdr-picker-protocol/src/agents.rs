use herdr_picker_domain::{Agent, AgentStatus, HerdrCall};
use serde_json::{Value, json};

use crate::wire::{ProtocolError, request_line};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireTab {
    pub tab_id: String,
    pub workspace_id: String,
    pub label: String,
}

pub fn agent_list_request(request_id: &str) -> String {
    request_line(request_id, "agent.list", json!({}))
}

pub fn tab_list_request(request_id: &str) -> String {
    request_line(request_id, "tab.list", json!({}))
}

pub fn call_request(request_id: &str, call: &HerdrCall) -> String {
    let (method, params) = match call {
        HerdrCall::FocusPane(id) => ("pane.focus", json!({ "pane_id": id })),
        HerdrCall::FocusTab(id) => ("tab.focus", json!({ "tab_id": id })),
        HerdrCall::FocusWorkspace(id) => ("workspace.focus", json!({ "workspace_id": id })),
        HerdrCall::CloseTab(id) => ("tab.close", json!({ "tab_id": id })),
        HerdrCall::Prompt { pane_id, text } => {
            ("agent.prompt", json!({ "target": pane_id, "text": text }))
        }
    };
    request_line(request_id, method, params)
}

pub fn agents_of(result: &Value) -> Result<Vec<Agent>, ProtocolError> {
    records(result, "agents")?
        .iter()
        .map(|record| {
            let text = |key| text_at(record, key);
            let kind = text("agent").unwrap_or_default();
            Ok(Agent {
                pane_id: text("pane_id").ok_or_else(|| missing("an agent", "pane_id"))?,
                workspace_id: text("workspace_id").unwrap_or_default(),
                tab_id: text("tab_id").unwrap_or_default(),
                title: text("terminal_title_stripped")
                    .filter(|title| !title.is_empty())
                    .unwrap_or_else(|| kind.clone()),
                kind,
                cwd: text("cwd").unwrap_or_default(),
                status: AgentStatus::parse(&text("agent_status").unwrap_or_default()),
                since: None,
            })
        })
        .collect()
}

pub fn tabs_of(result: &Value) -> Result<Vec<WireTab>, ProtocolError> {
    records(result, "tabs")?
        .iter()
        .map(|record| {
            Ok(WireTab {
                tab_id: text_at(record, "tab_id").ok_or_else(|| missing("a tab", "tab_id"))?,
                workspace_id: text_at(record, "workspace_id").unwrap_or_default(),
                label: text_at(record, "label").unwrap_or_default(),
            })
        })
        .collect()
}

// The pane and status a `pane.agent_status_changed` hook was run for.
pub fn parse_status_change(event_json: &str) -> Option<(String, AgentStatus)> {
    let event: Value = serde_json::from_str(event_json).ok()?;
    let data = event.get("data")?;
    let status = text_at(data, "agent_status")?;
    Some((text_at(data, "pane_id")?, AgentStatus::parse(&status)))
}

fn records<'a>(result: &'a Value, key: &str) -> Result<&'a Vec<Value>, ProtocolError> {
    result
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| ProtocolError::Malformed(format!("result has no {key} array")))
}

fn text_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn missing(what: &str, key: &str) -> ProtocolError {
    ProtocolError::Malformed(format!("{what} has no {key}"))
}

#[cfg(test)]
mod tests;
