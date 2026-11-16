use pwe_domain::test_engine::{OutputSpec, Report, RunCommands, RunnerSpec, RunnerStatus};
use serde::Deserialize;

use super::RunnerFileError;
use super::runner_file_check::Check;

const STATUSES: [&str; 2] = ["tested", "untested"];
const REPORTS: [&str; 2] = ["stdout", "file"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRunner {
    name: Option<String>,
    language: Option<String>,
    status: Option<String>,
    when: Option<String>,
    does: Option<String>,
    #[serde(default)]
    detect: RawDetect,
    list: Option<RawList>,
    #[serde(default)]
    run: RawRun,
    #[serde(default)]
    output: RawOutput,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawDetect {
    files: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawList {
    pub(super) argv: Option<Vec<String>>,
    pub(super) parser: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawRun {
    all: Option<Vec<String>>,
    ids: Option<Vec<String>>,
    file: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawOutput {
    parser: Option<String>,
    report: Option<String>,
}

pub fn read_runner_file(file: &str, text: &str) -> Result<RunnerSpec, RunnerFileError> {
    let raw: RawRunner = toml::from_str(text).map_err(|error| RunnerFileError::Unreadable {
        file: file.to_string(),
        reason: error.to_string(),
    })?;
    let check = Check { file };
    Ok(RunnerSpec {
        name: check.kebab_case("name", check.text("name", raw.name)?)?,
        language: check.text("language", raw.language)?,
        status: [RunnerStatus::Tested, RunnerStatus::Untested]
            [check.one_of("status", raw.status, &STATUSES)?],
        when: check.text("when", raw.when)?,
        does: check.text("does", raw.does)?,
        detect_files: check.words("detect.files", raw.detect.files)?,
        list: raw.list.map(|list| check.list_command(list)).transpose()?,
        run: RunCommands {
            all: check.words("run.all", raw.run.all)?,
            ids: check.optional_words("run.ids", raw.run.ids)?,
            file: check.optional_words("run.file", raw.run.file)?,
        },
        output: OutputSpec {
            parser: check.text("output.parser", raw.output.parser)?,
            report: [Report::Stdout, Report::File]
                [check.one_of("output.report", raw.output.report, &REPORTS)?],
        },
    })
}

#[cfg(test)]
mod tests;
