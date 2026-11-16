#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerStatus {
    Tested,
    Untested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    Stdout,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListCommand {
    pub argv: Vec<String>,
    pub parser: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommands {
    pub all: Vec<String>,
    pub ids: Option<Vec<String>>,
    pub file: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputSpec {
    pub parser: String,
    pub report: Report,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerSpec {
    pub name: String,
    pub language: String,
    pub status: RunnerStatus,
    pub when: String,
    pub does: String,
    pub detect_files: Vec<String>,
    pub list: Option<ListCommand>,
    pub run: RunCommands,
    pub output: OutputSpec,
}
