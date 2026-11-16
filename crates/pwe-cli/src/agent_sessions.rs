use pwe_adapters::ask_daemon;
use pwe_application::{AgentReadError, joined_sessions};
use pwe_protocol::{DaemonRequest, agent_sessions_json};

use crate::agent_hook;
use crate::command::{CommandError, daemon_socket, failed};
pub(crate) use crate::surroundings::Surroundings;

pub(crate) fn run() -> Result<(), CommandError> {
    if let Ok(listing) = ask_daemon(&daemon_socket(), DaemonRequest::Agents) {
        println!("{listing}");
        return Ok(());
    }
    let (printed, refusals) = listing(&Surroundings::of_this_process(), agent_hook::now())?;
    println!("{printed}");
    for refusal in refusals {
        eprintln!("pwe: {refusal}");
    }
    Ok(())
}

pub(crate) fn listing(
    surroundings: &Surroundings,
    now: u64,
) -> Result<(String, Vec<AgentReadError>), CommandError> {
    let sources = surroundings.read_agent_sources();
    let printed = agent_sessions_json(&joined_sessions(&sources, now)).map_err(failed)?;
    Ok((printed, sources.refusals))
}

#[cfg(test)]
mod tests;
