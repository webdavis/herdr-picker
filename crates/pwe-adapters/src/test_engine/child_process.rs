use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, PipeReader, Read};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use pwe_application::test_engine::{ToolProcess, ToolRun, ToolStartError};
use pwe_domain::test_engine::ToolEnd;
use rustix::process::{Pid, Signal, kill_process_group};

use crate::program::on_path;

const LONGEST_LINE_BYTES: u64 = 1024 * 1024;

pub struct ChildProcess {
    path_variable: Option<OsString>,
}

struct ChildRun {
    child: Child,
    output: BufReader<PipeReader>,
}

impl ChildProcess {
    pub fn new(path_variable: Option<OsString>) -> Self {
        ChildProcess { path_variable }
    }

    fn program(&self, name: &str) -> Result<PathBuf, ToolStartError> {
        if name.contains('/') {
            return Ok(PathBuf::from(name));
        }
        on_path(name, self.path_variable.as_deref())
            .ok_or_else(|| ToolStartError::Missing(name.to_string()))
    }
}

impl ToolProcess for ChildProcess {
    fn start(&mut self, argv: &[String], cwd: &str) -> Result<Box<dyn ToolRun>, ToolStartError> {
        let (name, arguments) = argv
            .split_first()
            .ok_or_else(|| ToolStartError::Missing(String::new()))?;
        let program = self.program(name)?;
        let refused = |failure: io::Error| ToolStartError::Failed {
            program: name.clone(),
            reason: failure.to_string(),
        };
        let (output, writer) = io::pipe().map_err(refused)?;
        let child = Command::new(program)
            .args(arguments)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stderr(writer.try_clone().map_err(refused)?)
            .stdout(writer)
            .process_group(0)
            .spawn()
            .map_err(refused)?;
        Ok(Box::new(ChildRun {
            child,
            output: BufReader::new(output),
        }))
    }
}

impl ToolRun for ChildRun {
    fn next_line(&mut self) -> Option<String> {
        let mut line = Vec::new();
        let read = (&mut self.output)
            .take(LONGEST_LINE_BYTES)
            .read_until(b'\n', &mut line)
            .ok()?;
        if read == 0 {
            return None;
        }
        let text = String::from_utf8_lossy(&line);
        Some(text.trim_end_matches(['\n', '\r']).to_string())
    }

    fn wait(&mut self) -> ToolEnd {
        ToolEnd::Exited {
            code: self.child.wait().ok().and_then(|status| status.code()),
        }
    }
}

impl Drop for ChildRun {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let group = i32::try_from(self.child.id()).ok().and_then(Pid::from_raw);
            if let Some(group) = group {
                let _ = kill_process_group(group, Signal::KILL);
            }
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests;
