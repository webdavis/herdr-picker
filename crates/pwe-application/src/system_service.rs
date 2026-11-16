use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemServiceError {
    ProgramMissing(String),
    ProgramRefused { program: String, detail: String },
    JobUnwritable(String),
}

impl fmt::Display for SystemServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProgramMissing(program) => {
                write!(formatter, "{program} was not found on PATH")
            }
            Self::ProgramRefused { program, detail } => {
                write!(formatter, "{program} refused: {detail}")
            }
            Self::JobUnwritable(detail) => {
                write!(formatter, "could not write the system job: {detail}")
            }
        }
    }
}

pub struct DaemonLaunch<'a> {
    pub program: &'a Path,
    pub path_variable: &'a str,
    pub log: &'a Path,
}

pub trait SystemService {
    fn enable(&self, launch: &DaemonLaunch) -> Result<(), SystemServiceError>;
    fn disable(&self) -> Result<(), SystemServiceError>;
    fn is_enabled(&self) -> bool;
}
