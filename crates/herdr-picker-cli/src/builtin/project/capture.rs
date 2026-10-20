use std::io::{self, Write};
use std::path::{Path, PathBuf};

use herdr_picker_adapters::run_capture;
use herdr_picker_domain::PickerRow;

use super::{command_row, nearest, root, words};
use crate::builtin::Places;
use crate::builtin::page::own_program;
use crate::builtin::paint::dim;
use crate::command::{CommandError, failed};

const MANIFESTS: [&str; 6] = [
    "Cargo.toml",
    "Package.swift",
    "package.json",
    "go.mod",
    "pytest.ini",
    "pyproject.toml",
];

// Where a project's saved checks live: one folder for the project the picker runs in.
pub(super) fn saved_dir(places: &Places) -> PathBuf {
    let scope = nearest(&places.cwd, &MANIFESTS)
        .and_then(|manifest| manifest.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| root(&places.cwd));
    places
        .captures
        .join(scope.to_string_lossy().replace('/', "%"))
}

// A check that runs only when asked, its output saved for the errors source to list.
pub(super) fn capture_row(
    label: &str,
    command: &str,
    cwd: &Path,
    framework: &str,
    places: &Places,
) -> PickerRow {
    let output = saved_dir(places).join(format!("diagnostics-{framework}.json"));
    let program = own_program();
    let saving = words(&[
        "capture",
        &output.to_string_lossy(),
        &cwd.to_string_lossy(),
        "diagnostics",
        framework,
        "--",
    ]);
    let detail = "Saves this run's results for the picker to read.\nIt may build code, run project scripts and reach the network, so it runs only when you ask.";
    let label = format!("{label}  {}", dim("save diagnostics"));
    command_row(label, format!("{program} {saving} {command}"), detail)
}

// herdr-picker capture <output> <cwd> <category> <framework> -- <command>...
pub(crate) fn save_check(arguments: &[&str]) -> Result<(), CommandError> {
    let [output, cwd, category, framework, "--", argv @ ..] = arguments else {
        return Err(CommandError::Usage);
    };
    let argv: Vec<String> = argv.iter().map(|word| word.to_string()).collect();
    let saved = run_capture(
        Path::new(output),
        Path::new(cwd),
        category,
        framework,
        &argv,
    )
    .map_err(failed)?;
    print!("{}", saved.stdout);
    let _ = io::stdout().flush();
    eprint!("{}", saved.stderr);
    eprintln!(
        "\nSaved {category} results. Reopen the picker to inspect them. Exit: {}",
        saved.exit_code
    );
    Ok(())
}
