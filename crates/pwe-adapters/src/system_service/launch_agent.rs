use std::fs;
use std::path::PathBuf;

use pwe_application::{DaemonLaunch, SystemService, SystemServiceError};
use pwe_protocol::{SystemJob, launchd_job};
use rustix::process::getuid;

use super::system_program::{run_system_program, unwritable};

const LAUNCHCTL: &str = "launchctl";

pub struct LaunchAgent {
    job_file: PathBuf,
}

impl LaunchAgent {
    pub fn at(job_file: PathBuf) -> Self {
        Self { job_file }
    }

    fn user_domain() -> String {
        format!("gui/{}", getuid().as_raw())
    }

    fn job_target() -> String {
        format!("{}/pwe-daemon", Self::user_domain())
    }

    fn remove_job_file(&self) -> Result<(), SystemServiceError> {
        match fs::remove_file(&self.job_file) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(unwritable(error)),
            _ => Ok(()),
        }
    }
}

impl SystemService for LaunchAgent {
    fn enable(&self, launch: &DaemonLaunch) -> Result<(), SystemServiceError> {
        if let Some(folder) = self.job_file.parent() {
            fs::create_dir_all(folder).map_err(unwritable)?;
        }
        let job = launchd_job(&SystemJob {
            program: launch.program,
            path_variable: launch.path_variable,
            log: launch.log,
        });
        fs::write(&self.job_file, job).map_err(unwritable)?;
        let _ = run_system_program(LAUNCHCTL, &["bootout", &Self::job_target()]);
        let job_file = self.job_file.to_string_lossy();
        run_system_program(LAUNCHCTL, &["bootstrap", &Self::user_domain(), &job_file]).inspect_err(
            |_| {
                let _ = self.remove_job_file();
            },
        )
    }

    fn disable(&self) -> Result<(), SystemServiceError> {
        let booted_out = run_system_program(LAUNCHCTL, &["bootout", &Self::job_target()]);
        self.remove_job_file()?;
        match booted_out {
            Err(SystemServiceError::ProgramMissing(program)) => {
                Err(SystemServiceError::ProgramMissing(program))
            }
            _ => Ok(()),
        }
    }

    fn is_enabled(&self) -> bool {
        self.job_file.exists()
    }
}
