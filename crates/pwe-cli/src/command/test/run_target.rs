use pwe_domain::test_engine::{Selection, Target, TestId};
use pwe_protocol::test_engine::{Hint, TestError, TestErrorCode, tool_test_name};

use super::flags::Flags;

const TARGETS: [&str; 3] = ["--all", "--file", "--id"];

pub(super) fn chosen_target(flags: &Flags) -> Result<Target, TestError> {
    let named = [flags.all, flags.file.is_some(), !flags.ids.is_empty()];
    if named.iter().filter(|named| **named).count() != 1 {
        return Err(bad_target("pwe test run takes exactly one target."));
    }
    if flags.all {
        return Ok(Target::All);
    }
    if let Some(file) = &flags.file {
        return Ok(Target::File(file.clone()));
    }
    flags
        .ids
        .iter()
        .map(|id| {
            id.parse::<TestId>().map_err(|_| {
                bad_target(&format!(
                    "`{id}` is not a test id; a test id reads runner:file::suite::name."
                ))
            })
        })
        .collect::<Result<_, _>>()
        .map(Target::Ids)
}

pub(super) fn selection(target: &Target, parser: &str) -> Selection {
    match target {
        Target::File(file) => Selection::File(file.clone()),
        Target::Ids(ids) => Selection::Names(
            ids.iter()
                .filter_map(|id| tool_test_name(parser, id))
                .collect(),
        ),
        _ => Selection::All,
    }
}

pub(super) fn bad_target(message: &str) -> TestError {
    TestError::new(TestErrorCode::BadTarget, message).with_hint(Hint {
        accepted: TARGETS.map(String::from).to_vec(),
        ..Hint::default()
    })
}
