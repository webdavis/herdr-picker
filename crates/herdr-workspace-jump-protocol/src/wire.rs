use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireWorkspace {
    pub workspace_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    Malformed(String),
    Server { code: String, message: String },
}

pub fn list_request(request_id: &str) -> String {
    request_line(request_id, "workspace.list", json!({}))
}

pub fn focus_request(request_id: &str, workspace_id: &str) -> String {
    request_line(
        request_id,
        "workspace.focus",
        json!({ "workspace_id": workspace_id }),
    )
}

pub fn create_request(request_id: &str, label: &str, cwd: &str) -> String {
    request_line(
        request_id,
        "workspace.create",
        json!({ "cwd": cwd, "label": label, "focus": true }),
    )
}

pub(crate) fn request_line(request_id: &str, method: &str, params: Value) -> String {
    let request = json!({ "id": request_id, "method": method, "params": params });
    format!("{request}\n")
}

pub fn result_of(body: &str) -> Result<Value, ProtocolError> {
    let envelope: Value = serde_json::from_str(body.trim())
        .map_err(|error| ProtocolError::Malformed(format!("not JSON: {error}")))?;
    if let Some(failure) = envelope.get("error") {
        return Err(ProtocolError::Server {
            code: string_at(failure, "code").unwrap_or_else(|| "unknown".to_string()),
            message: string_at(failure, "message").unwrap_or_else(|| body.trim().to_string()),
        });
    }
    match envelope.get("result") {
        Some(result) => Ok(result.clone()),
        None => Err(ProtocolError::Malformed(
            "envelope has neither result nor error".to_string(),
        )),
    }
}

pub fn workspaces_of(result: &Value) -> Result<Vec<WireWorkspace>, ProtocolError> {
    let records = result
        .get("workspaces")
        .and_then(Value::as_array)
        .ok_or_else(|| ProtocolError::Malformed("result has no workspaces array".to_string()))?;
    records
        .iter()
        .map(|record| {
            let workspace_id = string_at(record, "workspace_id").ok_or_else(|| {
                ProtocolError::Malformed("a workspace record has no workspace_id".to_string())
            })?;
            Ok(WireWorkspace {
                workspace_id,
                label: string_at(record, "label").unwrap_or_default(),
            })
        })
        .collect()
}

fn string_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests;
