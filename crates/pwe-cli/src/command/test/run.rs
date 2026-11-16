use std::env;
use std::path::Path;

use pwe_adapters::test_engine::{
    ChildProcess, RunFile, drop_old_runs, has_file, read_runner_files, repo_root, runner_folder,
};
use pwe_application::test_engine::{Clock, RunRequest, run_tests};
use pwe_domain::test_engine::{Retention, matched_runners};
use pwe_protocol::test_engine::{TestError, result_reader};

use super::event_printer::EventPrinter;
use super::flags::read_flags;
use super::history::runs_of;
use super::output::{chosen_format, exit_with, refused};
use super::run_again::resolved_target;
use super::run_runner::{chosen_runner, run_refusal, unknown_parser};
use super::run_target::{chosen_target, selection};
use super::system_clock::{SystemClock, new_run_id};
use crate::command::{CommandError, failed};

const ACCEPTED: [&str; 7] = [
    "--all", "--file", "--id", "--failed", "--last", "--runner", "--format",
];

pub(super) fn run(words: &[&str]) -> Result<(), CommandError> {
    let flags = read_flags(words, &ACCEPTED).map_err(|error| refused(None, &error))?;
    let format = chosen_format(flags.format.as_deref()).map_err(|error| refused(None, &error))?;
    let refuse = |error: TestError| refused(Some(format), &error);
    let root = repo_root(&env::current_dir().map_err(failed)?);
    let runs = runs_of(&root);
    let target = chosen_target(&flags)
        .and_then(|target| resolved_target(target, &runs))
        .map_err(refuse)?;
    let root_text = root.to_string_lossy().into_owned();
    let files = read_runner_files(&runner_folder(
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    ));
    let matched = matched_runners(&files.runners, |name| has_file(&root, name));
    let wanted = flags.runner.or(target.runner);
    let runner = chosen_runner(matched, wanted.as_deref(), &target.ran).map_err(refuse)?;
    let parser = &runner.spec.output.parser;
    let mut reader = result_reader(parser, &runner.spec.name, &root_text)
        .ok_or_else(|| refuse(unknown_parser(&runner)))?;
    let request = RunRequest {
        runner: &runner,
        selection: selection(&target.ran, parser),
        target: target.recorded,
        cwd: root_text.clone(),
        repo_root: root_text,
        commit: None,
    };
    let clock = SystemClock;
    let run_id = new_run_id(clock.now_ms());
    let mut printer = EventPrinter::new(format, &run_id, kept_run(&runs, &run_id));
    let verdict = run_tests(
        &request,
        reader.as_mut(),
        &mut ChildProcess::new(env::var_os("PATH")),
        &clock,
        &mut printer,
    )
    .map_err(|refusal| refuse(run_refusal(&runner, refusal)))?;
    printer.finish();
    let _ = drop_old_runs(&runs, &Retention::default());
    exit_with(verdict.exit_code)
}

fn kept_run(runs: &Path, run_id: &str) -> Option<RunFile> {
    RunFile::create(runs, run_id)
        .inspect_err(|failure| {
            eprintln!("pwe: this run is not kept in the test history: {failure}");
        })
        .ok()
}
