#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramName {
    ClaudeCode,
    Codex,
    Omp,
}

impl ProgramName {
    pub fn word(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Omp => "omp",
        }
    }

    pub fn parse(word: &str) -> Option<Self> {
        [Self::ClaudeCode, Self::Codex, Self::Omp]
            .into_iter()
            .find(|program| program.word() == word)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Working,
    Waiting,
    Finished,
    Idle,
    Failed,
    Ended,
    Unknown,
}

impl AgentState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Finished => "finished",
            Self::Idle => "idle",
            Self::Failed => "failed",
            Self::Ended => "ended",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitingKind {
    Permission,
    Question,
    Auth,
    Other,
}

impl WaitingKind {
    pub fn word(self) -> &'static str {
        match self {
            Self::Permission => "permission",
            Self::Question => "question",
            Self::Auth => "auth",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateFrom {
    Rex,
    Program,
}

impl StateFrom {
    pub fn word(self) -> &'static str {
        match self {
            Self::Rex => "rex",
            Self::Program => "program",
        }
    }
}
