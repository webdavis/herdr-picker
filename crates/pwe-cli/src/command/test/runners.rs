use std::env;

use pwe_adapters::test_engine::{has_file, read_runner_files, repo_root, runner_folder};
use pwe_domain::test_engine::matched_runners;
use pwe_protocol::test_engine::{runner_listing_json, runner_listing_text};

use super::flags::read_flags;
use super::output::{chosen_format, encoded, print_out, refused};
use crate::command::{CommandError, failed};

const ACCEPTED: [&str; 1] = ["--format"];

pub(super) fn runners(words: &[&str]) -> Result<(), CommandError> {
    let flags = read_flags(words, &ACCEPTED).map_err(|error| refused(None, &error))?;
    let format = chosen_format(flags.format.as_deref()).map_err(|error| refused(None, &error))?;
    let root = repo_root(&env::current_dir().map_err(failed)?);
    let files = read_runner_files(&runner_folder(
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    ));
    let matched = matched_runners(&files.runners, |name| has_file(&root, name));
    print_out(&encoded(
        format,
        &runner_listing_json(&matched, &files.broken),
        || runner_listing_text(&matched, &files.broken),
    ));
    Ok(())
}
