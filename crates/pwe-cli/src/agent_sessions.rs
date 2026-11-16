use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use pwe_adapters::{
    ClaudeCodeSessions, CodexSessions, LsofOpenFiles, OmpSessions, RexPanes, RexSettings,
    SystemProcessEnvironment, read_picker_config, rex_program,
};
use pwe_application::{AgentReadError, PaneStatuses, agent_sessions};
use pwe_protocol::agent_sessions_json;

use crate::agent_hook;
use crate::command::{CommandError, config_path, failed};

pub(crate) struct Surroundings {
    pub(crate) path_variable: Option<OsString>,
    pub(crate) home: PathBuf,
    pub(crate) codex_folder: PathBuf,
    pub(crate) proc_folder: PathBuf,
    pub(crate) applications: PathBuf,
    pub(crate) rex_app_name: String,
}

impl Surroundings {
    pub(crate) fn of_this_process() -> Self {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        Self {
            path_variable: env::var_os("PATH"),
            codex_folder: env::var_os("CODEX_HOME")
                .map_or_else(|| home.join(".codex"), PathBuf::from),
            home,
            proc_folder: PathBuf::from("/proc"),
            applications: PathBuf::from("/Applications"),
            rex_app_name: read_picker_config(&config_path()).map_or_else(
                |_| RexSettings::default().app_name,
                |config| config.rex.app_name,
            ),
        }
    }

    pub(crate) fn rex_program(&self) -> Option<PathBuf> {
        rex_program(
            self.path_variable.as_deref(),
            &self.applications,
            &self.rex_app_name,
        )
        .ok()
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
    let mut rex = surroundings
        .rex_program()
        .map(|binary| RexPanes::at(binary, None));
    let mut rex_panes: Vec<&mut dyn PaneStatuses> = rex
        .iter_mut()
        .map(|panes| panes as &mut dyn PaneStatuses)
        .collect();
    let (sessions, refusals) = agent_sessions(
        &mut [&mut claude_code, &mut omp, &mut codex],
        &mut rex_panes,
        now,
    );
    let printed = agent_sessions_json(&sessions).map_err(failed)?;
    Ok((printed, refusals))
}

#[cfg(test)]
mod tests;
