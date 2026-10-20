use herdr_picker_domain::Size;
use serde_json::{Map, Value, json};

use crate::wire::{ProtocolError, request_line};

pub const SOCKET_PATH_VARIABLE: &str = "HERDR_SOCKET_PATH";
pub const ACTIVE_WORKSPACE_VARIABLE: &str = "HERDR_ACTIVE_WORKSPACE_ID";
pub const PICKER_NAME_VARIABLE: &str = "HERDR_PICKER_NAME";
// herdr overwrites HERDR_PLUGIN_CONFIG_DIR in a plugin pane,
// so the config file the caller read travels under its own name.
pub const PICKER_CONFIG_VARIABLE: &str = "HERDR_PICKER_CONFIG";
pub const PICKER_RESULT_VARIABLE: &str = "HERDR_PICKER_RESULT";
pub const PICKER_BASE_VARIABLE: &str = "HERDR_PICKER_BASE";
pub const PICKER_TARGET_VARIABLE: &str = "HERDR_PICKER_TARGET";
pub(crate) const WORKSPACE_PICKER_PANE: &str = "pick";
pub(crate) const LAUNCHER_PANE: &str = "launcher";
pub(crate) const CONFIGURED_PICKER_PANE: &str = "run_picker";

pub struct PopupPane {
    entrypoint: &'static str,
    env: Map<String, Value>,
    cwd: Option<String>,
}

impl PopupPane {
    pub fn workspace_picker(current_workspace: Option<&str>) -> Self {
        Self {
            entrypoint: WORKSPACE_PICKER_PANE,
            env: current_workspace
                .map(|workspace_id| (ACTIVE_WORKSPACE_VARIABLE.to_string(), workspace_id.into()))
                .into_iter()
                .collect(),
            cwd: None,
        }
    }

    pub fn launcher(base: &str, target: &str, cwd: Option<&str>) -> Self {
        Self {
            entrypoint: LAUNCHER_PANE,
            env: [
                (PICKER_BASE_VARIABLE, base),
                (PICKER_TARGET_VARIABLE, target),
            ]
            .into_iter()
            .map(|(variable, value)| (variable.to_string(), value.into()))
            .collect(),
            cwd: cwd.map(str::to_string),
        }
    }

    pub fn configured_picker(name: &str, config: &str, result: &str, cwd: Option<&str>) -> Self {
        Self {
            entrypoint: CONFIGURED_PICKER_PANE,
            env: [
                (PICKER_NAME_VARIABLE, name),
                (PICKER_CONFIG_VARIABLE, config),
                (PICKER_RESULT_VARIABLE, result),
            ]
            .into_iter()
            .map(|(variable, value)| (variable.to_string(), value.into()))
            .collect(),
            cwd: cwd.map(str::to_string),
        }
    }
}

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

pub fn popup_request(request_id: &str, plugin_id: &str, size: Size, pane: &PopupPane) -> String {
    let mut params = json!({
        "plugin_id": plugin_id,
        "entrypoint": pane.entrypoint,
        "placement": "popup",
        "width": size.columns,
        "height": size.rows,
        "env": pane.env,
    });
    if let Some(cwd) = &pane.cwd {
        params["cwd"] = cwd.as_str().into();
    }
    request_line(request_id, "plugin.pane.open", params)
}

#[cfg(test)]
mod tests;
