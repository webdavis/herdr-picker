use super::*;
use crate::test_engine::{OutputSpec, Report, RunCommands, RunnerSpec, RunnerStatus};

fn words(text: &[&str]) -> Vec<String> {
    text.iter().map(|word| word.to_string()).collect()
}

fn runner(run: RunCommands) -> RunnerSpec {
    RunnerSpec {
        name: "probe".to_string(),
        language: "rust".to_string(),
        status: RunnerStatus::Tested,
        when: "always".to_string(),
        does: "probes".to_string(),
        detect_files: words(&["Cargo.toml"]),
        list: None,
        run,
        output: OutputSpec {
            parser: "junit".to_string(),
            report: Report::File,
        },
    }
}

fn cargo() -> RunnerSpec {
    runner(RunCommands {
        all: words(&["cargo", "test", "--no-fail-fast"]),
        ids: Some(words(&["cargo", "test", "--", "--exact", "{names...}"])),
        file: None,
    })
}

fn argv(spec: &RunnerSpec, selection: &Selection) -> Result<Vec<String>, InvocationError> {
    invocation(spec, selection, None).map(|invocation| invocation.argv)
}

#[test]
fn each_name_becomes_one_argument_even_with_spaces_and_quotes() {
    let names = words(&["tests::has space", "tests::\"quoted\"", "tests::it's"]);

    assert_eq!(
        argv(&cargo(), &Selection::Names(names)),
        Ok(words(&[
            "cargo",
            "test",
            "--",
            "--exact",
            "tests::has space",
            "tests::\"quoted\"",
            "tests::it's",
        ]))
    );
}

#[test]
fn everything_runs_the_all_command_as_written() {
    assert_eq!(
        argv(&cargo(), &Selection::All),
        Ok(words(&["cargo", "test", "--no-fail-fast"]))
    );
}

#[test]
fn a_file_fills_its_placeholder_inside_an_argument() {
    let spec = runner(RunCommands {
        all: words(&["pytest"]),
        ids: None,
        file: Some(words(&[
            "pytest",
            "--rootdir=.",
            "{file}",
            "--junitxml={report}",
        ])),
    });

    assert_eq!(
        invocation(
            &spec,
            &Selection::File("tests/test a.py".to_string()),
            Some("/tmp/report.xml")
        )
        .map(|invocation| invocation.argv),
        Ok(words(&[
            "pytest",
            "--rootdir=.",
            "tests/test a.py",
            "--junitxml=/tmp/report.xml",
        ]))
    );
}

#[test]
fn a_runner_without_a_file_command_cannot_run_a_file() {
    assert_eq!(
        argv(&cargo(), &Selection::File("src/lib.rs".to_string())),
        Err(InvocationError::NoFileCommand)
    );
}

#[test]
fn a_runner_without_an_ids_command_cannot_run_names() {
    let spec = runner(RunCommands {
        all: words(&["make", "test"]),
        ids: None,
        file: None,
    });

    assert_eq!(
        argv(&spec, &Selection::Names(words(&["a"]))),
        Err(InvocationError::NoIdsCommand)
    );
}

#[test]
fn a_report_placeholder_without_a_report_path_is_refused() {
    let spec = runner(RunCommands {
        all: words(&["swift", "test", "--xunit-output", "{report}"]),
        ids: None,
        file: None,
    });

    assert_eq!(
        argv(&spec, &Selection::All),
        Err(InvocationError::Unfilled("report".to_string()))
    );
}

#[test]
fn an_unknown_placeholder_is_refused_naming_it() {
    let spec = runner(RunCommands {
        all: words(&["go", "test", "{package}"]),
        ids: None,
        file: None,
    });

    assert_eq!(
        argv(&spec, &Selection::All),
        Err(InvocationError::Unknown("package".to_string()))
    );
}

#[test]
fn braces_that_are_not_a_placeholder_stay_as_written() {
    let spec = runner(RunCommands {
        all: words(&["grep", "-E", "a{2}", "{}"]),
        ids: None,
        file: None,
    });

    assert_eq!(
        argv(&spec, &Selection::All),
        Ok(words(&["grep", "-E", "a{2}", "{}"]))
    );
}
