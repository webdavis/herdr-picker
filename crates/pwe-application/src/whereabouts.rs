use std::fmt;

use pwe_domain::OwnerPlace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereaboutsError {
    PermissionRefused { app_name: String },
    ProgramMissing(String),
    Unreadable(String),
}

impl fmt::Display for WhereaboutsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionRefused { app_name } => write!(
                formatter,
                "macOS refused pwe permission to ask {app_name} which session is shown"
            ),
            Self::ProgramMissing(program) => write!(formatter, "{program} was not found on PATH"),
            Self::Unreadable(detail) => {
                write!(
                    formatter,
                    "could not read which Rex session is shown: {detail}"
                )
            }
        }
    }
}

pub trait Whereabouts {
    fn owner_place(&self) -> Result<OwnerPlace, WhereaboutsError>;
}
