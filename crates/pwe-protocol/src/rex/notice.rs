use pwe_domain::{NoticePlace, Size};
use serde_json::{Value, json};

use super::pane::parsed;
use super::window::window_cells;
use crate::ProtocolError;

const TERMINAL_FLAVOR: &str = "com.superlogical.terminal.shell";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedLayer {
    pub layer: String,
    pub block: String,
}

pub fn session_request(session: &str) -> String {
    json!({"session_id": session}).to_string()
}

pub fn focused_block_in_view(view: &str) -> Result<String, ProtocolError> {
    let view = parsed(view, "session.view")?;
    let active = &view["active_window_id"];
    view["windows"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|window| &window["window_id"] == active)
        .and_then(|window| window["focused_block_id"].as_str())
        .map(str::to_string)
        .ok_or_else(|| ProtocolError::Malformed("Rex named no focused block".to_string()))
}

pub fn window_in_view(view: &str, block: &str, size: &str) -> Result<Size, ProtocolError> {
    let facts = json!({"view": parsed(view, "session.view")?, "size": parsed(size, "size")?});
    Ok(window_cells(&facts, block))
}

pub fn notice_float_request(
    session: &str,
    place: &NoticePlace,
    line: &str,
    command: &[String],
) -> String {
    json!({
        "session_id": session,
        "focus": false,
        "bounds": place.bounds,
        "layout": {"block": {
            "flavor": TERMINAL_FLAVOR,
            "label": line,
            "options": {
                "size": {"columns": place.size.columns, "rows": place.size.rows},
                "command": command,
            },
        }},
    })
    .to_string()
}

pub fn opened_layer(answer: &str) -> Result<OpenedLayer, ProtocolError> {
    let answer = parsed(answer, "session.new_layer")?;
    let text = |pointer| answer.pointer(pointer).and_then(Value::as_str);
    match (text("/layer_id"), text("/block_ids/0")) {
        (Some(layer), Some(block)) => Ok(OpenedLayer {
            layer: layer.to_string(),
            block: block.to_string(),
        }),
        _ => Err(ProtocolError::Malformed(
            "Rex named no new layer and block".to_string(),
        )),
    }
}

pub fn block_label_request(session: &str, block: &str, label: &str) -> String {
    json!({"session_id": session, "block_id": block, "label": label}).to_string()
}

pub fn focus_block_request(session: &str, block: &str) -> String {
    json!({"session_id": session, "block_id": block}).to_string()
}

pub fn close_layer_request(session: &str, layer: &str) -> String {
    json!({"session_id": session, "layer_id": layer}).to_string()
}

#[cfg(test)]
mod tests;
