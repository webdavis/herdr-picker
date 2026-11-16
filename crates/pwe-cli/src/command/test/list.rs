use std::env;

use pwe_adapters::test_engine::{ChildProcess, repo_root, save_listing};
use pwe_application::test_engine::{ListTestsError, RunTestsError, list_tests};
use pwe_domain::test_engine::{MatchedRunner, Target, ToolEnd};
use pwe_protocol::test_engine::{
    Hint, LIST_PARSERS, Listing, TestError, TestErrorCode, listing_json, listing_text,
    read_listing, test_json,
};

use super::flags::read_flags;
use super::history::runs_of;
use super::output::{Format, chosen_format, encoded, exit_with, print_out, refused};
use super::run_runner::{chosen_runner, run_refusal};
use super::runners::folder_runners;
use crate::command::{CommandError, failed};

const ACCEPTED: [&str; 2] = ["--runner", "--format"];
const LISTED_EXIT_CODE: i32 = 0;
const BUILD_FAILED_EXIT_CODE: i32 = 3;

pub(super) fn list(words: &[&str]) -> Result<(), CommandError> {
    let flags = read_flags(words, &ACCEPTED).map_err(|error| refused(None, &error))?;
    let format = chosen_format(flags.format.as_deref()).map_err(|error| refused(None, &error))?;
    let refuse = |error: TestError| refused(Some(format), &error);
    let root = repo_root(&env::current_dir().map_err(failed)?);
    let root_text = root.to_string_lossy().into_owned();
    let (matched, _) = folder_runners(&root);
    let runner = chosen_runner(matched, flags.runner.as_deref(), &Target::All).map_err(refuse)?;
    let output = list_tests(
        &runner,
        &mut ChildProcess::new(env::var_os("PATH")),
        &root_text,
    )
    .map_err(|refusal| refuse(list_refusal(&runner, refusal)))?;
    let parser = runner
        .spec
        .list
        .as_ref()
        .map_or("", |list| list.parser.as_str());
    let listing = read_listing(parser, &runner.spec.name, &root_text, &output.lines)
        .ok_or_else(|| refuse(unknown_list_parser(&runner, parser)))?;
    let built = listing.build_failed.is_none() && output.end == ToolEnd::Exited { code: Some(0) };
    if built
        && let Err(failure) =
            save_listing(&runs_of(&root), &runner.spec.name, &test_lines(&listing))
    {
        eprintln!("pwe: this listing is not saved: {failure}");
    }
    print_out(&shown(format, &listing));
    exit_with(match built {
        true => LISTED_EXIT_CODE,
        false => BUILD_FAILED_EXIT_CODE,
    })
}

fn test_lines(listing: &Listing) -> String {
    listing
        .tests
        .iter()
        .map(|id| format!("{}\n", test_json(id)))
        .collect()
}

fn shown(format: Format, listing: &Listing) -> String {
    match format {
        Format::Jsonl if listing.build_failed.is_none() => test_lines(listing),
        _ => encoded(format, &listing_json(listing), || listing_text(listing)),
    }
}

fn list_refusal(runner: &MatchedRunner, refusal: ListTestsError) -> TestError {
    match refusal {
        ListTestsError::NoListCommand => TestError::new(
            TestErrorCode::NoTests,
            format!(
                "The {} runner has no [list] command that lists its tests.",
                runner.spec.name
            ),
        ),
        ListTestsError::Start(start) => run_refusal(runner, RunTestsError::Start(start)),
    }
}

fn unknown_list_parser(runner: &MatchedRunner, parser: &str) -> TestError {
    TestError::new(
        TestErrorCode::BadRunnerFile,
        format!(
            "The {} runner's list.parser `{parser}` is not a parser pwe knows.",
            runner.spec.name
        ),
    )
    .with_hint(Hint {
        accepted: LIST_PARSERS.map(String::from).to_vec(),
        ..Hint::default()
    })
}
