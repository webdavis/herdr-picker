mod reading;

use std::fmt;

use serde_json::Value;

use reading::session_event;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RexBlock {
    pub rex_session: String,
    pub block: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramStatus {
    pub state: String,
    pub kind: Option<String>,
    pub program: String,
    pub message: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundProcess {
    pub pid: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RexEvent {
    TabChanged {
        rex_session: String,
        tab: String,
    },
    ViewChanged {
        rex_session: String,
    },
    ProgramStatusChanged {
        at: RexBlock,
        status: ProgramStatus,
    },
    ProgramStatusRemoved {
        at: RexBlock,
    },
    ProcessChanged {
        at: RexBlock,
        foreground: Option<ForegroundProcess>,
    },
    BlockClosed {
        at: RexBlock,
    },
    ChildExited {
        at: RexBlock,
        exit_code: Option<i64>,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RexEventError {
    pub line_number: usize,
    pub line: String,
}

impl fmt::Display for RexEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "line {} of Rex's events is not JSON: {}",
            self.line_number, self.line
        )
    }
}

pub fn parse_rex_event(line_number: usize, line: &str) -> Result<RexEvent, RexEventError> {
    let event: Value = serde_json::from_str(line).map_err(|_| RexEventError {
        line_number,
        line: line.trim().to_string(),
    })?;
    Ok(session_event(&event).unwrap_or(RexEvent::Other))
}

#[cfg(test)]
mod tests;
