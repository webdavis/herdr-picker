use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use pwe_adapters::{
    AgentSettings, ClaudeCodeSessions, CodexSessions, LsofOpenFiles, OmpSessions, RexPanes,
    RexSettings, SystemProcessEnvironment, read_picker_config, rex_program,
};
use pwe_application::{AgentProgram, AgentSources, PaneStatuses, read_agent_sources};
use pwe_domain::ProgramName;

use crate::command::config_path;

pub(crate) struct Surroundings {
    pub(crate) path_variable: Option<OsString>,
    pub(crate) home: PathBuf,
    pub(crate) codex_folder: PathBuf,
    pub(crate) proc_folder: PathBuf,
    pub(crate) applications: PathBuf,
    pub(crate) rex: RexSettings,
    pub(crate) agents: AgentSettings,
}

impl Surroundings {
    pub(crate) fn of_this_process() -> Self {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let (rex, agents) = read_picker_config(&config_path()).map_or_else(
            |_| (RexSettings::default(), AgentSettings::default()),
            |config| (config.rex, config.agents),
        );
        Self {
            path_variable: env::var_os("PATH"),
            codex_folder: env::var_os("CODEX_HOME")
                .map_or_else(|| home.join(".codex"), PathBuf::from),
            home,
            proc_folder: PathBuf::from("/proc"),
            applications: PathBuf::from("/Applications"),
            rex,
            agents,
        }
    }

    pub(crate) fn rex_program(&self) -> Option<PathBuf> {
        rex_program(
            self.path_variable.as_deref(),
            &self.applications,
            &self.rex.app_name,
        )
        .ok()
    }

    pub(crate) fn read_agent_sources(&self) -> AgentSources {
        let path = self.path_variable.as_deref();
        let environment = SystemProcessEnvironment::on(path, &self.proc_folder);
        let mut claude_code = ClaudeCodeSessions::on(path, environment);
        let mut omp = OmpSessions::on(path, &self.home.join(".omp"), LsofOpenFiles::on(path));
        let mut codex = CodexSessions::on(&self.codex_folder, LsofOpenFiles::on(path));
        let chosen = self.agents.programs();
        let mut programs: Vec<&mut dyn AgentProgram> = Vec::new();
        for (name, program) in [
            (
                ProgramName::ClaudeCode,
                &mut claude_code as &mut dyn AgentProgram,
            ),
            (ProgramName::Omp, &mut omp),
            (ProgramName::Codex, &mut codex),
        ] {
            if chosen.contains(&name) {
                programs.push(program);
            }
        }
        let mut rex = self.rex_program().map(|binary| RexPanes::at(binary, None));
        let mut rex_panes: Vec<&mut dyn PaneStatuses> = rex
            .iter_mut()
            .map(|panes| panes as &mut dyn PaneStatuses)
            .collect();
        read_agent_sources(&mut programs, &mut rex_panes)
    }
}
