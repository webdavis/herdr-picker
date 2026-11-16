mod agent_hook;
mod agent_sessions;
mod builtin;
mod command;
mod generate;
mod handing;
mod inbox;
mod last_workspace;
mod launch;
mod path;
mod pick;
mod picker;
mod picker_chrome;
mod picker_keys;
mod picker_popup;
mod picker_step;
mod popup;
mod quickfix;
mod record;
mod rex;
mod run;

use std::env;
use std::process::ExitCode;

use command::{CommandError, USAGE};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match command::execute(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CommandError::Usage) => {
            eprintln!("pwe: {USAGE}");
            ExitCode::from(2)
        }
        Err(CommandError::Failed(reason)) => {
            eprintln!("pwe: {reason}");
            ExitCode::FAILURE
        }
        Err(CommandError::Exit(code)) => ExitCode::from(code),
    }
}

#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
