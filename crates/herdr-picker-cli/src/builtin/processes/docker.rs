use herdr_picker_domain::{Action, PickerRow};
use serde_json::Value;

use super::services::{service_row, text};
use crate::builtin::page::words;
use crate::builtin::rows::key;

const DOCKER_FIELDS: [&str; 9] = [
    "ID", "Name", "Names", "Image", "State", "Status", "Ports", "Service", "Project",
];

// `docker ps` or `docker compose ps` as JSON lines or an array; containers in `seen` are skipped.
pub(super) fn docker_rows(listed: &str, seen: &mut Vec<String>) -> Result<Vec<PickerRow>, String> {
    let items: Vec<Value> = match listed.trim_start().starts_with('[') {
        true => serde_json::from_str(listed).map_err(|why| why.to_string())?,
        false => listed
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()
            .map_err(|why| why.to_string())?,
    };
    let mut rows = Vec::new();
    for item in items {
        let id = [&item["ID"], &item["Id"]].map(text).concat();
        if id.is_empty() || seen.contains(&id) {
            continue;
        }
        seen.push(id.clone());
        let name = [&item["Names"], &item["Name"]]
            .into_iter()
            .map(text)
            .find(|name| !name.is_empty());
        let detail = DOCKER_FIELDS
            .iter()
            .filter(|field| !item[**field].is_null())
            .map(|field| format!("{field}: {}", text(&item[*field])))
            .collect::<Vec<_>>()
            .join("\n");
        let status = [&item["State"], &item["Status"]]
            .into_iter()
            .map(text)
            .find(|status| !status.is_empty());
        let format = "{{.Name}}  {{.State.Status}}  {{.Config.Image}}";
        let inspect = ["docker", "inspect", "--format", format, &id];
        let mut row = service_row(
            &name.unwrap_or(id.clone()),
            &status.unwrap_or_default(),
            &inspect,
            &detail,
            "Docker",
        );
        row.keys.push((
            key("alt-d"),
            Action::Insert(words(&["docker", "logs", "--tail", "200", &id])),
        ));
        row.keys.push((
            key("alt-e"),
            Action::Insert(words(&["docker", "exec", "-it", &id, "sh"])),
        ));
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests;
