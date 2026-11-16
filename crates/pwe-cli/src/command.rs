mod agents;
mod daemon;
mod doctor;
mod inbox;
mod notice;
mod pickers;
mod rex;
mod test;
mod tools;
mod workspaces;

use std::env;
use std::path::PathBuf;

use pwe_adapters::{config_file, daemon_socket_file, plugin_state_dir};
use pwe_protocol::PICKER_CONFIG_VARIABLE;

pub(crate) const USAGE: &str = "usage: pwe jump <label> <cwd> [--after-pid <pid>] | pwe pick | pwe popup | pwe last-workspace | pwe record | pwe agent-changed | pwe launch <base> | pwe launcher [<base> --under <block>] | pwe deliver <picker> <pane> --after-pid <pid> | pwe generate --output <directory> [--config <file>] | pwe run <picker> | pwe run-in-popup | pwe preview <rows-file> <index> | pwe header <rows-file> | pwe rows <rows-file> | pwe live <rows-file> <query> | pwe view <rows-file> load|choose|back|key <key> <index>... | pwe footer <rows-file> [index] | pwe help <rows-file> [index] | pwe builtin <name> [args] | pwe git-preview <diff|stages|blob> <args> | pwe capture <output> <cwd> <category> <framework> -- <command>... | pwe network ping <address> <page> | pwe network wake <mac> <broadcast>... | pwe tabs focus <browser> <window> <tab> [space] | pwe init bash|rex | pwe daemon run|start|stop|enable|disable | pwe daemon status [--format json|text] | pwe agents | pwe notice --level needs-you|finished --until <seconds since 1970> | pwe rex key <base> | pwe doctor [--format json|text] (exits 0 when every check passes, 1 when one fails) | pwe test runners [--format json|jsonl|toon|text] | pwe test run --all|--file <file>|--id <id>... [--runner <name>] [--format json|jsonl|toon|text]";

#[derive(Debug)]
pub(crate) enum CommandError {
    Usage,
    Failed(String),
    Exit(u8),
}

pub(crate) fn failed(refusal: impl std::fmt::Display) -> CommandError {
    CommandError::Failed(refusal.to_string())
}

pub(crate) fn prints_json(options: &[&str]) -> Result<bool, CommandError> {
    match options {
        [] | ["--format", "text"] => Ok(false),
        ["--format", "json"] => Ok(true),
        _ => Err(CommandError::Usage),
    }
}

pub(crate) fn config_path() -> PathBuf {
    config_file(
        env::var("HERDR_PLUGIN_CONFIG_DIR").ok().as_deref(),
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

pub(crate) fn picker_config_path() -> PathBuf {
    env::var_os(PICKER_CONFIG_VARIABLE).map_or_else(config_path, PathBuf::from)
}

pub(crate) fn state_dir() -> PathBuf {
    plugin_state_dir(
        env::var("HERDR_PLUGIN_STATE_DIR").ok().as_deref(),
        env::var("XDG_STATE_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

pub(crate) fn daemon_socket() -> PathBuf {
    daemon_socket_file(
        env::var("XDG_RUNTIME_DIR").ok().as_deref(),
        env::var("XDG_STATE_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

pub(crate) fn execute(arguments: &[String]) -> Result<(), CommandError> {
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    workspaces::execute(&words)
        .or_else(|| rex::execute(&words))
        .or_else(|| pickers::execute(&words))
        .or_else(|| tools::execute(&words))
        .or_else(|| inbox::execute(&words))
        .or_else(|| daemon::execute(&words))
        .or_else(|| agents::execute(&words))
        .or_else(|| doctor::execute(&words))
        .or_else(|| test::execute(&words))
        .or_else(|| notice::execute(&words))
        .unwrap_or(Err(CommandError::Usage))
}

#[cfg(test)]
mod tests;
