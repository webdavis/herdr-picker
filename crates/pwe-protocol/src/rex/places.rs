use pwe_domain::Pane;
use serde_json::Value;

use super::pane::parsed;
use crate::ProtocolError;

const UNNAMED_TAB_PREFIX: &str = "Window ";
const ID_SUFFIX_LENGTH: usize = 8;

pub fn focused_pane_in_view(session: &str, view: &str) -> Result<Pane, ProtocolError> {
    let view = parsed(view, "session.view")?;
    let active = &view["active_window_id"];
    tabs(&view)
        .find(|tab| &tab["window_id"] == active)
        .and_then(|tab| pane_in(session, tab, tab["focused_block_id"].as_str()?))
        .ok_or_else(|| ProtocolError::Malformed("Rex named no focused pane".to_string()))
}

pub fn placed_pane_in_view(
    session: &str,
    view: &str,
    block: &str,
) -> Result<Option<(Pane, String)>, ProtocolError> {
    let view = parsed(view, "session.view")?;
    Ok(tabs(&view)
        .enumerate()
        .find(|(_, tab)| blocks_of(tab).any(|held| held["block_id"] == block))
        .and_then(|(index, tab)| Some((pane_in(session, tab, block)?, tab_name(tab, index)))))
}

fn tab_name(tab: &Value, index: usize) -> String {
    let label = tab["label"].as_str().unwrap_or_default();
    let id = tab["window_id"].as_str().unwrap_or_default();
    let named_by_rex_from_its_id = label
        .strip_prefix(UNNAMED_TAB_PREFIX)
        .is_some_and(|suffix| suffix.len() == ID_SUFFIX_LENGTH && id.ends_with(suffix));
    match named_by_rex_from_its_id {
        true => (index + 1).to_string(),
        false => label.to_string(),
    }
}

fn tabs(view: &Value) -> impl Iterator<Item = &Value> {
    view["windows"].as_array().into_iter().flatten()
}

fn blocks_of(tab: &Value) -> impl Iterator<Item = &Value> {
    tab["layers"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|layer| layer["blocks"].as_array().into_iter().flatten())
}

fn pane_in(session: &str, tab: &Value, block: &str) -> Option<Pane> {
    Some(Pane {
        rex_session: session.to_string(),
        tab: tab["window_id"].as_str()?.to_string(),
        block: block.to_string(),
    })
}

#[cfg(test)]
mod tests;
