use std::path::PathBuf;

use herdr_picker_adapters::{read_config, write_manifest};
use herdr_picker_domain::launcher_bases;

use crate::command::{CommandError, config_path, failed};

struct Options {
    output: PathBuf,
    config: Option<PathBuf>,
}

pub(crate) fn run(arguments: &[&str]) -> Result<(), CommandError> {
    let options = parse(arguments)?;
    let config = options.config.unwrap_or_else(config_path);
    let config = read_config(&config).map_err(failed)?;
    let bases = launcher_bases(&config.pickers);
    write_manifest(
        &options.output,
        env!("CARGO_PKG_VERSION"),
        &config.targets,
        &bases,
    )
    .map_err(failed)
}

fn parse(arguments: &[&str]) -> Result<Options, CommandError> {
    let mut output = None;
    let mut config = None;
    let mut remaining = arguments;
    while let [flag, value, rest @ ..] = remaining {
        let slot = match *flag {
            "--output" => &mut output,
            "--config" => &mut config,
            _ => return Err(CommandError::Usage),
        };
        if slot.is_some() || is_a_flag(value) {
            return Err(CommandError::Usage);
        }
        *slot = Some(PathBuf::from(value));
        remaining = rest;
    }
    if !remaining.is_empty() {
        return Err(CommandError::Usage);
    }
    Ok(Options {
        output: output.ok_or(CommandError::Usage)?,
        config,
    })
}

fn is_a_flag(word: &str) -> bool {
    word.starts_with("--")
}

#[cfg(test)]
mod tests;
