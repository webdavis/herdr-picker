use std::fs;
use std::path::PathBuf;

use pwe_application::{DaemonLaunch, SystemService, SystemServiceError};
use pwe_protocol::{SystemJob, systemd_unit};

use super::system_program::{run_system_program, unwritable};

const SYSTEMCTL: &str = "systemctl";
const UNIT: &str = "pwe-daemon.service";

pub struct SystemdUser {
    unit_file: PathBuf,
}

impl SystemdUser {
    pub fn at(unit_file: PathBuf) -> Self {
        Self { unit_file }
    }

    fn remove_unit_file(&self) -> Result<(), SystemServiceError> {
        match fs::remove_file(&self.unit_file) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(unwritable(error)),
            _ => Ok(()),
        }
    }
}

impl SystemService for SystemdUser {
    fn enable(&self, launch: &DaemonLaunch) -> Result<(), SystemServiceError> {
        if let Some(folder) = self.unit_file.parent() {
            fs::create_dir_all(folder).map_err(unwritable)?;
        }
        let unit = systemd_unit(&SystemJob {
            program: launch.program,
            path_variable: launch.path_variable,
            log: launch.log,
        });
        fs::write(&self.unit_file, unit).map_err(unwritable)?;
        run_system_program(SYSTEMCTL, &["--user", "daemon-reload"])
            .and_then(|()| run_system_program(SYSTEMCTL, &["--user", "enable", "--now", UNIT]))
            .inspect_err(|_| {
                let _ = self.remove_unit_file();
            })
    }

    fn disable(&self) -> Result<(), SystemServiceError> {
        let disabled = run_system_program(SYSTEMCTL, &["--user", "disable", "--now", UNIT]);
        self.remove_unit_file()?;
        let _ = run_system_program(SYSTEMCTL, &["--user", "daemon-reload"]);
        match disabled {
            Err(SystemServiceError::ProgramMissing(program)) => {
                Err(SystemServiceError::ProgramMissing(program))
            }
            _ => Ok(()),
        }
    }

    fn is_enabled(&self) -> bool {
        self.unit_file.exists()
    }
}
