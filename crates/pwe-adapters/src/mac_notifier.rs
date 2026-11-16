use std::ffi::OsString;

use pwe_application::{DesktopNotifier, DesktopNotifierError, DesktopRoute};

use crate::program::on_path;
use crate::program_run::{ProgramRunError, run_on_path};

const OSASCRIPT: &str = "osascript";
const TERMINAL_NOTIFIER: &str = "terminal-notifier";

pub struct MacNotifier {
    route: DesktopRoute,
    path_variable: Option<OsString>,
}

impl MacNotifier {
    pub fn new(route: DesktopRoute, path_variable: Option<OsString>) -> Self {
        Self {
            route,
            path_variable,
        }
    }

    fn chosen_program(&self) -> Option<&'static str> {
        match self.route {
            DesktopRoute::Off => None,
            DesktopRoute::Osascript => Some(OSASCRIPT),
            DesktopRoute::TerminalNotifier => Some(TERMINAL_NOTIFIER),
            DesktopRoute::Auto
                if on_path(TERMINAL_NOTIFIER, self.path_variable.as_deref()).is_some() =>
            {
                Some(TERMINAL_NOTIFIER)
            }
            DesktopRoute::Auto => Some(OSASCRIPT),
        }
    }

    fn missing(&self, program: &str) -> DesktopNotifierError {
        match self.route {
            DesktopRoute::Auto => DesktopNotifierError::NoProgram,
            _ => DesktopNotifierError::ProgramMissing(program.to_string()),
        }
    }
}

fn arguments<'a>(program: &str, title: &'a str, text: &'a str) -> Vec<&'a str> {
    match program {
        TERMINAL_NOTIFIER => vec!["-title", title, "-message", text],
        _ => vec![
            "-e",
            "on run argv",
            "-e",
            "display notification (item 2 of argv) with title (item 1 of argv)",
            "-e",
            "end run",
            "--",
            title,
            text,
        ],
    }
}

impl DesktopNotifier for MacNotifier {
    fn notify(&self, title: &str, text: &str) -> Result<(), DesktopNotifierError> {
        let Some(program) = self.chosen_program() else {
            return Ok(());
        };
        let refused = |detail: String| DesktopNotifierError::Refused {
            program: program.to_string(),
            detail,
        };
        let output = run_on_path(
            program,
            self.path_variable.as_deref(),
            &arguments(program, title, text),
            "",
        )
        .map_err(|failure| match failure {
            ProgramRunError::Missing => self.missing(program),
            ProgramRunError::Unrunnable(error) => refused(error.to_string()),
        })?;
        match output.status.success() {
            true => Ok(()),
            false => Err(refused(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests;
