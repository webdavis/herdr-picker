use std::fmt;
use std::path::Path;

use pwe_application::{DaemonLaunch, SystemService, SystemServiceError};

use crate::lifecycle::{LifecycleError, stop_daemon, wait_for_answer};

#[derive(Debug)]
pub enum RegistrationError {
    Service(SystemServiceError),
    Lifecycle(LifecycleError),
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Service(error) => write!(formatter, "{error}"),
            Self::Lifecycle(error) => write!(formatter, "{error}"),
        }
    }
}

impl From<SystemServiceError> for RegistrationError {
    fn from(error: SystemServiceError) -> Self {
        Self::Service(error)
    }
}

impl From<LifecycleError> for RegistrationError {
    fn from(error: LifecycleError) -> Self {
        Self::Lifecycle(error)
    }
}

pub fn enable_daemon(
    service: &dyn SystemService,
    launch: &DaemonLaunch,
    socket: &Path,
) -> Result<u32, RegistrationError> {
    stop_daemon(socket)?;
    service.enable(launch)?;
    Ok(wait_for_answer(socket, launch.log, || false)?)
}

pub fn disable_daemon(service: &dyn SystemService, socket: &Path) -> Result<(), RegistrationError> {
    service.disable()?;
    stop_daemon(socket)?;
    Ok(())
}
