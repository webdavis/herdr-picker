use std::env;

use pwe_adapters::test_engine::{
    ChildProcess, has_file, read_runner_files, repo_root, runner_folder,
};
use pwe_application::test_engine::{Clock, RunRequest, run_tests};
use pwe_domain::test_engine::matched_runners;
use pwe_protocol::test_engine::{TestError, result_reader};

use super::event_printer::EventPrinter;
use super::flags::read_flags;
use super::output::{chosen_format, refused};
use super::run_runner::{chosen_runner, run_refusal, unknown_parser};
use super::run_target::{chosen_target, selection};
use super::system_clock::{SystemClock, new_run_id};
use crate::command::{CommandError, failed};

const ACCEPTED: [&str; 5] = ["--all", "--file", "--id", "--runner", "--format"];

pub(super) fn run(words: &[&str]) -> Result<(), CommandError> {
    let flags = read_flags(words, &ACCEPTED).map_err(|error| refused(None, &error))?;
    let format = chosen_format(flags.format.as_deref()).map_err(|error| refused(None, &error))?;
    let refuse = |error: TestError| refused(Some(format), &error);
    let target = chosen_target(&flags).map_err(refuse)?;
    let root = repo_root(&env::current_dir().map_err(failed)?);
    let root_text = root.to_string_lossy().into_owned();
    let files = read_runner_files(&runner_folder(
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    ));
    let matched = matched_runners(&files.runners, |name| has_file(&root, name));
    let runner = chosen_runner(matched, flags.runner.as_deref(), &target).map_err(refuse)?;
    let parser = &runner.spec.output.parser;
    let mut reader = result_reader(parser, &runner.spec.name, &root_text)
        .ok_or_else(|| refuse(unknown_parser(&runner)))?;
    let request = RunRequest {
        runner: &runner,
        selection: selection(&target, parser),
        target,
        cwd: root_text.clone(),
        repo_root: root_text,
        commit: None,
    };
    let clock = SystemClock;
    let mut printer = EventPrinter::new(format, &new_run_id(clock.now_ms()));
    let verdict = run_tests(
        &request,
        reader.as_mut(),
        &mut ChildProcess::new(env::var_os("PATH")),
        &clock,
        &mut printer,
    )
    .map_err(|refusal| refuse(run_refusal(&runner, refusal)))?;
    printer.finish();
    match u8::try_from(verdict.exit_code) {
        Ok(0) => Ok(()),
        Ok(code) => Err(CommandError::Exit(code)),
        Err(_) => Err(CommandError::Exit(1)),
    }
}
