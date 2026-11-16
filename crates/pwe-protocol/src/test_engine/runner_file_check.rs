use pwe_domain::test_engine::ListCommand;

use super::RunnerFileError;
use super::runner_file::RawList;

pub(super) struct Check<'a> {
    pub(super) file: &'a str,
}

impl Check<'_> {
    pub(super) fn text(
        &self,
        field: &'static str,
        value: Option<String>,
    ) -> Result<String, RunnerFileError> {
        let value = value.ok_or_else(|| self.missing(field))?;
        if value.trim().is_empty() {
            return Err(self.empty(field));
        }
        Ok(value)
    }

    pub(super) fn words(
        &self,
        field: &'static str,
        value: Option<Vec<String>>,
    ) -> Result<Vec<String>, RunnerFileError> {
        self.optional_words(field, value)?
            .ok_or_else(|| self.missing(field))
    }

    pub(super) fn optional_words(
        &self,
        field: &'static str,
        value: Option<Vec<String>>,
    ) -> Result<Option<Vec<String>>, RunnerFileError> {
        match value {
            Some(words) if words.is_empty() => Err(self.empty(field)),
            other => Ok(other),
        }
    }

    pub(super) fn list_command(&self, list: RawList) -> Result<ListCommand, RunnerFileError> {
        Ok(ListCommand {
            argv: self.words("list.argv", list.argv)?,
            parser: self.text("list.parser", list.parser)?,
        })
    }

    pub(super) fn one_of(
        &self,
        field: &'static str,
        value: Option<String>,
        accepted: &'static [&'static str],
    ) -> Result<usize, RunnerFileError> {
        let value = self.text(field, value)?;
        accepted
            .iter()
            .position(|candidate| *candidate == value)
            .ok_or_else(|| RunnerFileError::NotOneOf {
                file: self.file.to_string(),
                field,
                value,
                accepted,
            })
    }

    pub(super) fn kebab_case(
        &self,
        field: &'static str,
        value: String,
    ) -> Result<String, RunnerFileError> {
        let is_kebab_case = value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        });
        if is_kebab_case {
            return Ok(value);
        }
        Err(RunnerFileError::NotKebabCase {
            file: self.file.to_string(),
            field,
            value,
        })
    }

    fn missing(&self, field: &'static str) -> RunnerFileError {
        RunnerFileError::Missing {
            file: self.file.to_string(),
            field,
        }
    }

    fn empty(&self, field: &'static str) -> RunnerFileError {
        RunnerFileError::Empty {
            file: self.file.to_string(),
            field,
        }
    }
}
