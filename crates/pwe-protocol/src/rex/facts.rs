use pwe_domain::Size;
use serde_json::Value;

use super::window::window_cells;
use crate::ProtocolError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyFacts {
    pub session: String,
    pub block: String,
    pub label: String,
    pub program: String,
    pub folder: String,
    pub window: Size,
}

pub fn parse_key_facts(text: &str) -> Result<KeyFacts, ProtocolError> {
    let facts: Value = serde_json::from_str(text)
        .map_err(|failure| ProtocolError::Malformed(format!("Rex key facts: {failure}")))?;
    let block = text_at(&facts, "/block")
        .ok_or_else(|| ProtocolError::Malformed("Rex key facts name no block".to_string()))?;
    let process = facts
        .pointer("/process/foreground")
        .or_else(|| facts.pointer("/process/child"))
        .unwrap_or(&Value::Null);
    Ok(KeyFacts {
        session: text_at(&facts, "/view/session_id").unwrap_or_default(),
        label: label_of(&facts, &block).unwrap_or_default(),
        program: text_at(process, "/name").unwrap_or_default(),
        folder: text_at(process, "/cwd").unwrap_or_default(),
        window: window_cells(&facts, &block),
        block,
    })
}

fn label_of(facts: &Value, block: &str) -> Option<String> {
    facts
        .pointer("/view/windows")?
        .as_array()?
        .iter()
        .filter_map(|window| window["layers"].as_array())
        .flatten()
        .filter_map(|layer| layer["blocks"].as_array())
        .flatten()
        .find(|listed| listed["block_id"] == block)
        .and_then(|listed| text_at(listed, "/label"))
}

fn text_at(value: &Value, pointer: &str) -> Option<String> {
    value.pointer(pointer)?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests;
