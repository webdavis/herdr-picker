use herdr_workspace_jump_domain::Size;
use serde_json::{Map, Value, json};

use crate::wire::{ProtocolError, request_line};

pub const ACTIVE_WORKSPACE_VARIABLE: &str = "HERDR_ACTIVE_WORKSPACE_ID";
pub(crate) const PICKER_PANE: &str = "pick";

pub fn layout_request(request_id: &str) -> String {
    request_line(request_id, "pane.layout", json!({}))
}

pub fn window_of(result: &Value) -> Result<Size, ProtocolError> {
    Ok(Size {
        columns: area_side(result, "width")?,
        rows: area_side(result, "height")?,
    })
}

fn area_side(result: &Value, side: &str) -> Result<usize, ProtocolError> {
    result
        .pointer(&format!("/layout/area/{side}"))
        .and_then(Value::as_u64)
        .and_then(|cells| usize::try_from(cells).ok())
        .ok_or_else(|| ProtocolError::Malformed(format!("the layout has no area {side}")))
}

pub fn picker_request(
    request_id: &str,
    plugin_id: &str,
    size: Size,
    current_workspace: Option<&str>,
) -> String {
    let env: Map<String, Value> = current_workspace
        .map(|workspace_id| (ACTIVE_WORKSPACE_VARIABLE.to_string(), workspace_id.into()))
        .into_iter()
        .collect();
    request_line(
        request_id,
        "plugin.pane.open",
        json!({
            "plugin_id": plugin_id,
            "entrypoint": PICKER_PANE,
            "placement": "popup",
            "width": size.columns,
            "height": size.rows,
            "env": env,
        }),
    )
}

#[cfg(test)]
mod tests;
