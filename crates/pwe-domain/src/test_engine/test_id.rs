use std::fmt;
use std::str::FromStr;

const SEPARATOR: &str = "::";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestId {
    pub runner: String,
    pub file: String,
    pub suite: String,
    pub name: String,
    pub case: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestIdError {
    NoRunner,
    NoFile,
    NoName,
}

impl FromStr for TestId {
    type Err = TestIdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (runner, located) = split_runner(text)?;
        let (file, named) = located.split_once(SEPARATOR).ok_or(TestIdError::NoName)?;
        if file.is_empty() {
            return Err(TestIdError::NoFile);
        }
        let (path, case) = split_case(named);
        let (suite, name) = path.rsplit_once(SEPARATOR).unwrap_or(("", path));
        if name.is_empty() {
            return Err(TestIdError::NoName);
        }
        Ok(TestId {
            runner: runner.to_string(),
            file: file.to_string(),
            suite: suite.to_string(),
            name: name.to_string(),
            case: case.map(str::to_string),
        })
    }
}

fn split_runner(text: &str) -> Result<(&str, &str), TestIdError> {
    match text.split_once(':') {
        Some((runner, rest)) if !runner.is_empty() && !rest.starts_with(':') => Ok((runner, rest)),
        Some((runner, rest)) if !runner.is_empty() && rest.starts_with(SEPARATOR) => {
            Ok((runner, rest))
        }
        _ => Err(TestIdError::NoRunner),
    }
}

fn split_case(named: &str) -> (&str, Option<&str>) {
    match (named.find('['), named.strip_suffix(']')) {
        (Some(open), Some(closed)) => (&named[..open], Some(&closed[open + 1..])),
        _ => (named, None),
    }
}

impl fmt::Display for TestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}{SEPARATOR}", self.runner, self.file)?;
        if !self.suite.is_empty() {
            write!(formatter, "{}{SEPARATOR}", self.suite)?;
        }
        write!(formatter, "{}", self.name)?;
        match &self.case {
            Some(case) => write!(formatter, "[{case}]"),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
