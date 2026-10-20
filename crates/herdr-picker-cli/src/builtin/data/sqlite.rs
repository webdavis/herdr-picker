use std::path::Path;
use std::time::Duration;

use herdr_picker_domain::PickerRow;
use serde::Deserialize;

use crate::builtin::git::plural;
use crate::builtin::page::{command_row, notice, words};
use crate::builtin::paint::dim;
use crate::builtin::program::run;

const DEADLINE: Duration = Duration::from_secs(5);
const SCHEMA: &str = "SELECT type, name, tbl_name, sql FROM sqlite_schema \
    WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name";
const COLUMNS: &str = "SELECT m.name AS owner, p.name, p.type, p.\"notnull\" AS required, p.pk \
    FROM sqlite_schema m, pragma_table_info(m.name) p WHERE m.type IN ('table', 'view') \
    ORDER BY m.name, p.cid";

#[derive(Deserialize)]
struct Item {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    tbl_name: String,
    sql: Option<String>,
}

#[derive(Deserialize)]
struct Column {
    owner: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    required: u8,
    pk: u8,
}

// A database's tables, views and indexes, each a query to prepare or run, read-only throughout.
pub(super) fn tables(path: &Path) -> Vec<PickerRow> {
    let database = path.to_string_lossy();
    let (items, columns) = match (
        asked::<Item>(&database, SCHEMA),
        asked::<Column>(&database, COLUMNS),
    ) {
        (Ok(items), Ok(columns)) => (items, columns),
        (Err(failure), _) | (_, Err(failure)) => {
            return vec![notice("Not a SQLite database sqlite3 can read", &failure)];
        }
    };
    items
        .iter()
        .map(|item| {
            let quoted = format!("\"{}\"", item.name.replace('"', "\"\""));
            let query = match item.kind.as_str() {
                "table" | "view" => format!("SELECT * FROM {quoted} LIMIT 100;"),
                _ => format!(
                    "SELECT sql FROM sqlite_schema WHERE name = '{}';",
                    item.name.replace('\'', "''")
                ),
            };
            let own: Vec<&Column> = columns
                .iter()
                .filter(|column| column.owner == item.name)
                .collect();
            let indexes: Vec<&str> = items
                .iter()
                .filter(|index| index.kind == "index" && index.tbl_name == item.name)
                .map(|index| index.name.as_str())
                .collect();
            let mut detail = item.sql.clone().unwrap_or_else(|| item.name.clone());
            if !own.is_empty() {
                let lines: Vec<String> = own.iter().map(|column| described(column)).collect();
                detail.push_str(&format!("\n\nColumns:\n{}", lines.join("\n")));
            }
            if !indexes.is_empty() {
                detail.push_str(&format!("\n\nIndexes: {}", indexes.join(", ")));
            }
            let label = match own.len() {
                0 => item.name.clone(),
                count => format!("{}  {}", item.name, dim(&plural(count, "column"))),
            };
            let command = words(&[
                "sqlite3",
                "-readonly",
                "-header",
                "-column",
                &database,
                &query,
            ]);
            PickerRow {
                group: Some(item.kind.clone()),
                ..command_row(label, command, &detail)
            }
        })
        .collect()
}

fn described(column: &Column) -> String {
    let required = if column.required == 1 {
        "  NOT NULL"
    } else {
        ""
    };
    let key = if column.pk > 0 { "  PRIMARY KEY" } else { "" };
    format!("{}  {}{required}{key}", column.name, column.kind)
}

fn asked<T: for<'de> Deserialize<'de>>(database: &str, query: &str) -> Result<Vec<T>, String> {
    let printed = run(
        &["sqlite3", "-readonly", "-json", database, query],
        DEADLINE,
    )?;
    match printed.trim().is_empty() {
        true => Ok(Vec::new()),
        false => serde_json::from_str(&printed).map_err(|failure| failure.to_string()),
    }
}

#[cfg(test)]
mod tests;
