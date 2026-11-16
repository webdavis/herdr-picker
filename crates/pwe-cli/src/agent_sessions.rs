use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use pwe_adapters::{
    ClaudeCodeSessions, CodexSessions, LsofOpenFiles, OmpSessions, SystemProcessEnvironment,
};
use pwe_application::{AgentReadError, agent_sessions};
use pwe_protocol::agent_sessions_json;

use crate::agent_hook;
use crate::command::{CommandError, failed};

pub(crate) struct Surroundings {
    pub(crate) path_variable: Option<OsString>,
    pub(crate) home: PathBuf,
    pub(crate) codex_folder: PathBuf,
    pub(crate) proc_folder: PathBuf,
}

impl Surroundings {
    fn of_this_process() -> Self {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        Self {
            path_variable: env::var_os("PATH"),
            codex_folder: env::var_os("CODEX_HOME")
                .map_or_else(|| home.join(".codex"), PathBuf::from),
            home,
            proc_folder: PathBuf::from("/proc"),
        }
    }
}

pub(crate) fn run() -> Result<(), CommandError> {
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
    let path = surroundings.path_variable.as_deref();
    let environment = SystemProcessEnvironment::on(path, &surroundings.proc_folder);
    let mut claude_code = ClaudeCodeSessions::on(path, environment);
    let mut omp = OmpSessions::on(
        path,
        &surroundings.home.join(".omp"),
        LsofOpenFiles::on(path),
    );
    let mut codex = CodexSessions::on(&surroundings.codex_folder, LsofOpenFiles::on(path));
    let (sessions, refusals) = agent_sessions(&mut [&mut claude_code, &mut omp, &mut codex], now);
    let printed = agent_sessions_json(&sessions).map_err(failed)?;
    Ok((printed, refusals))
}

#[cfg(test)]
mod tests;
