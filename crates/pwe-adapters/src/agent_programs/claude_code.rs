use std::ffi::OsStr;
use std::path::PathBuf;

use pwe_application::{AgentProgram, AgentReadError, ProcessEnvironment};
use pwe_domain::{ProgramName, ProgramReport};
use pwe_protocol::claude_code_sessions;

use super::printed::{printed_by, unreadable};
use crate::program::on_path;

const CONFIG_FOLDER_VARIABLE: &str = "CLAUDE_CONFIG_DIR";

pub struct ClaudeCodeSessions<E> {
    claude: Option<PathBuf>,
    environment: E,
}

impl<E: ProcessEnvironment> ClaudeCodeSessions<E> {
    pub fn on(path_variable: Option<&OsStr>, environment: E) -> Self {
        Self {
            claude: on_path("claude", path_variable),
            environment,
        }
    }
}

impl<E: ProcessEnvironment> AgentProgram for ClaudeCodeSessions<E> {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError> {
        let Some(claude) = &self.claude else {
            return Ok(Vec::new());
        };
        let printed = printed_by(claude, &["agents", "--json"]).map_err(refusal)?;
        let mut sessions =
            claude_code_sessions(&printed).map_err(|error| refusal(unreadable(error)))?;
        for session in &mut sessions {
            session.account = session
                .pid
                .and_then(|pid| self.environment.variable(pid, CONFIG_FOLDER_VARIABLE));
        }
        Ok(sessions)
    }
}

fn refusal(detail: String) -> AgentReadError {
    AgentReadError {
        program: ProgramName::ClaudeCode.word().to_string(),
        detail,
    }
}

#[cfg(test)]
mod tests;
