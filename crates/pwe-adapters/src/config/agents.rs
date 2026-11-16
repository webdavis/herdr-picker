use pwe_domain::ProgramName;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum AgentProgramWord {
    ClaudeCode,
    Codex,
    Omp,
}

impl From<AgentProgramWord> for ProgramName {
    fn from(word: AgentProgramWord) -> Self {
        match word {
            AgentProgramWord::ClaudeCode => Self::ClaudeCode,
            AgentProgramWord::Codex => Self::Codex,
            AgentProgramWord::Omp => Self::Omp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct AgentSettings {
    programs: Vec<AgentProgramWord>,
    pub refresh_seconds: u64,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            programs: vec![
                AgentProgramWord::ClaudeCode,
                AgentProgramWord::Codex,
                AgentProgramWord::Omp,
            ],
            refresh_seconds: 15,
        }
    }
}

impl AgentSettings {
    pub fn programs(&self) -> Vec<ProgramName> {
        self.programs
            .iter()
            .copied()
            .map(ProgramName::from)
            .collect()
    }
}
