use std::env;
use std::io::{self, Read};

use pwe_adapters::{RexSettings, read_picker_config};
use pwe_domain::{key_passes_on, launcher_base};
use pwe_protocol::{ProtocolError, float_request, parse_key_facts};

use crate::command::{CommandError, config_path, failed};

pub(crate) const UNDER: &str = "--under";

pub(crate) fn key(base: &str) -> Result<(), CommandError> {
    let mut facts = String::new();
    io::stdin().read_to_string(&mut facts).map_err(failed)?;
    let settings = read_picker_config(&config_path()).map_err(failed)?.rex;
    let pwe = env::current_exe().map_err(failed)?;
    if let Some(request) = answer_key(base, &facts, &settings, &pwe.to_string_lossy())? {
        print!("{request}");
    }
    Ok(())
}

pub(crate) fn answer_key(
    base: &str,
    facts: &str,
    settings: &RexSettings,
    pwe: &str,
) -> Result<Option<String>, CommandError> {
    launcher_base(base).ok_or(CommandError::Usage)?;
    let facts = parse_key_facts(facts).map_err(|failure| match failure {
        ProtocolError::Malformed(detail) => failed(detail),
        ProtocolError::Server { code, message } => failed(format!("{code}: {message}")),
    })?;
    if key_passes_on(&facts.label, &facts.program, &settings.pass_keys_to) {
        return Ok(None);
    }
    let command = [pwe, "launcher", base, UNDER, &facts.block].map(str::to_string);
    Ok(Some(float_request(&facts, &command)))
}

#[cfg(test)]
mod tests;
