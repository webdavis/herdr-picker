use std::path::Path;

use pwe_domain::test_engine::{Diagnostic, TestEvent, UntrustedText, printed_name};
use serde::Deserialize;

use super::severity_words::severity_of;

const COMPILER_MESSAGE: &str = "compiler-message";
const BUILD_FINISHED: &str = "build-finished";

pub(super) struct BuildDiagnostics {
    repo_root: String,
    found: Vec<Diagnostic>,
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<CompilerMessage>,
    success: Option<bool>,
}

#[derive(Deserialize)]
struct CompilerMessage {
    level: String,
    message: String,
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Span {
    file_name: String,
    line_start: u32,
    column_start: u32,
    is_primary: bool,
}

impl BuildDiagnostics {
    pub(super) fn new(repo_root: &str) -> Self {
        BuildDiagnostics {
            repo_root: repo_root.to_string(),
            found: Vec::new(),
        }
    }

    pub(super) fn read(&mut self, line: &str) -> Option<TestEvent> {
        if !line.starts_with('{') {
            return None;
        }
        let message: CargoMessage = serde_json::from_str(line).ok()?;
        match (message.reason.as_str(), message.message, message.success) {
            (COMPILER_MESSAGE, Some(compiler), _) => {
                self.record(compiler);
                None
            }
            (BUILD_FINISHED, _, Some(false)) => Some(TestEvent::BuildFailed {
                diagnostics: std::mem::take(&mut self.found),
            }),
            _ => None,
        }
    }

    fn record(&mut self, compiler: CompilerMessage) {
        let Some(span) = compiler.spans.into_iter().find(|span| span.is_primary) else {
            return;
        };
        let diagnostic = Diagnostic {
            file: printed_name(&self.named_from_root(span.file_name)),
            line: span.line_start,
            column: span.column_start,
            severity: severity_of(&compiler.level),
            message: UntrustedText::new(&compiler.message),
        };
        if !self.found.contains(&diagnostic) {
            self.found.push(diagnostic);
        }
    }

    fn named_from_root(&self, file: String) -> String {
        Path::new(&file)
            .strip_prefix(&self.repo_root)
            .ok()
            .and_then(Path::to_str)
            .map_or(file.clone(), str::to_string)
    }
}

#[cfg(test)]
mod tests;
