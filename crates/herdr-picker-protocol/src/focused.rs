use serde_json::Value;

fn text_at(json: &str, pointer: &str) -> Option<String> {
    serde_json::from_str::<Value>(json)
        .ok()?
        .pointer(pointer)?
        .as_str()
        .map(str::to_string)
}

pub fn parse_focused_id(event_json: &str) -> Option<String> {
    text_at(event_json, "/data/workspace_id")
}

pub fn parse_focused_pane(event_json: &str) -> Option<String> {
    text_at(event_json, "/data/pane_id")
}

// herdr hands a plugin action the pane focused when its key was pressed, and that pane's folder.
pub fn parse_context_pane(context_json: &str) -> Option<(String, Option<String>)> {
    let pane = text_at(context_json, "/focused_pane_id")?;
    Some((pane, text_at(context_json, "/focused_pane_cwd")))
}

#[cfg(test)]
mod tests;
