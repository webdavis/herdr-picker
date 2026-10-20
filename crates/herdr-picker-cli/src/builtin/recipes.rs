use std::io::Write;
use std::path::Path;
use std::process::Command;

use herdr_picker_domain::{Action, PickerRow, Preview};
use herdr_picker_protocol::shell_word;
use serde_json::Value;

use super::git::output;
use super::paint::dim;
use super::rows::{RowWriter, in_home, key};
use super::{Places, written};
use crate::command::{CommandError, failed};

// One recipe as just dumps it, with the command that runs it from anywhere.
pub(super) struct Recipe {
    pub(super) name: String,
    pub(super) command: String,
    pub(super) doc: String,
    parameters: String,
    pub(super) required: bool,
    shown: String,
    source: String,
}

pub(super) fn recipes_rows(
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let recipes = recipes(&places.cwd)?;
    let width = recipes.iter().map(|recipe| recipe.name.len()).max();
    written(
        recipes.iter().try_for_each(|recipe| {
            rows.row(&recipe_row(recipe, width.unwrap_or_default(), places))
        }),
    )
}

pub(super) fn recipes(cwd: &Path) -> Result<Vec<Recipe>, CommandError> {
    let dump =
        output(
            Command::new("just")
                .current_dir(cwd)
                .args(["--dump", "--dump-format", "json"]),
        )?;
    let dump: Value = serde_json::from_slice(&dump).map_err(failed)?;
    let source = dump["source"].as_str().unwrap_or_default().to_string();
    let mut found = Vec::new();
    visit(&dump, &mut found);
    found
        .into_iter()
        .map(|(name, recipe)| {
            let shown = output(Command::new("just").current_dir(cwd).args([
                "--justfile",
                &source,
                "--show",
                &name,
            ]))?;
            Ok(Recipe {
                command: ["just", "--justfile", &source, &name]
                    .map(shell_word)
                    .join(" "),
                doc: recipe["doc"].as_str().unwrap_or_default().to_string(),
                parameters: parameters(&recipe),
                required: recipe["parameters"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|parameter| parameter["default"].is_null() && parameter["kind"] != "star"),
                shown: String::from_utf8_lossy(&shown).into_owned(),
                source: source.clone(),
                name,
            })
        })
        .collect()
}

// The module's public recipes, then each submodule's under its path.
fn visit(module: &Value, found: &mut Vec<(String, Value)>) {
    let recipes = module["recipes"].as_object().into_iter().flatten();
    for recipe in recipes.map(|(_, recipe)| recipe) {
        if recipe["private"] != true {
            let name = recipe["namepath"].as_str().unwrap_or_default().to_string();
            found.push((name, recipe.clone()));
        }
    }
    for child in module["modules"].as_object().into_iter().flatten() {
        visit(child.1, found);
    }
}

fn parameters(recipe: &Value) -> String {
    let shown = recipe["parameters"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|parameter| {
            let name = parameter["name"].as_str().unwrap_or_default();
            match &parameter["default"] {
                Value::String(default) => format!("{name}={default}"),
                Value::Null if parameter["kind"] == "star" => name.to_string(),
                Value::Null => format!("{name} (required)"),
                _ => format!("{name} (default from an expression)"),
            }
        });
    shown.collect::<Vec<_>>().join(", ")
}

pub(super) fn recipe_row(recipe: &Recipe, width: usize, places: &Places) -> PickerRow {
    let details = [
        ("Runs", format!("just {}", recipe.name)),
        ("Parameters", recipe.parameters.clone()),
        (
            "Defined in",
            in_home(Path::new(&recipe.source), places.home.as_deref()),
        ),
    ];
    let details: Vec<String> = details
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(field, value)| format!("{field:<10}  {value}"))
        .collect();
    // Its arguments are typed after it at the prompt, so it is only prepared.
    let (enter, keys) = match recipe.required {
        true => (Action::Insert(format!("{} ", recipe.command)), Vec::new()),
        false => (
            Action::Insert(recipe.command.clone()),
            vec![(key("ctrl-enter"), Action::Run(recipe.command.clone()))],
        ),
    };
    PickerRow {
        label: match recipe.doc.is_empty() {
            true => recipe.name.clone(),
            false => format!("{:<width$}  {}", recipe.name, dim(&recipe.doc)),
        },
        preview: Some(Preview::Text(format!(
            "{}\n\n{}",
            details.join("\n"),
            recipe.shown
        ))),
        enter,
        keys,
        group: None,
    }
}

#[cfg(test)]
pub(crate) mod tests;
