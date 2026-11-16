use std::fmt;

use pwe_domain::{ProgramName, RecordedPane};

const LONGEST_SESSION_ID: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoResumeLine {
    ProgramHasNone,
    UnsafeSessionId,
}

impl fmt::Display for NoResumeLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ProgramHasNone => "the program has no resume command",
            Self::UnsafeSessionId => "the recorded session id is not a plain id",
        })
    }
}

pub fn resume_line(recorded: &RecordedPane) -> Result<String, NoResumeLine> {
    let session_id = &recorded.session_id;
    let resume = match recorded.program {
        ProgramName::ClaudeCode => format!("claude --resume {session_id}"),
        ProgramName::Codex => format!("codex resume {session_id}"),
        ProgramName::Omp => return Err(NoResumeLine::ProgramHasNone),
    };
    if !is_a_plain_session_id(session_id) {
        return Err(NoResumeLine::UnsafeSessionId);
    }
    Ok(resume)
}

fn is_a_plain_session_id(session_id: &str) -> bool {
    (1..=LONGEST_SESSION_ID).contains(&session_id.len())
        && session_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
}
