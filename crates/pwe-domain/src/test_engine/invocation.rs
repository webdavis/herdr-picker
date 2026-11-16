use super::RunnerSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    File(String),
    Names(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvocationError {
    NoFileCommand,
    NoIdsCommand,
    Unfilled(String),
    Unknown(String),
}

const NAMES: &str = "{names...}";

struct Values<'a> {
    file: Option<&'a str>,
    names: Option<&'a [String]>,
    report: Option<&'a str>,
}

pub fn invocation(
    spec: &RunnerSpec,
    selection: &Selection,
    report: Option<&str>,
) -> Result<Invocation, InvocationError> {
    let (template, values) = match selection {
        Selection::All => (&spec.run.all, Values::none(report)),
        Selection::File(file) => (
            spec.run
                .file
                .as_ref()
                .ok_or(InvocationError::NoFileCommand)?,
            Values {
                file: Some(file),
                ..Values::none(report)
            },
        ),
        Selection::Names(names) => (
            spec.run.ids.as_ref().ok_or(InvocationError::NoIdsCommand)?,
            Values {
                names: Some(names),
                ..Values::none(report)
            },
        ),
    };
    let mut argv = Vec::new();
    for argument in template {
        if argument == NAMES {
            argv.extend(values.names()?.iter().cloned());
        } else {
            argv.push(fill(argument, &values)?);
        }
    }
    Ok(Invocation { argv })
}

impl<'a> Values<'a> {
    fn none(report: Option<&'a str>) -> Self {
        Values {
            file: None,
            names: None,
            report,
        }
    }

    fn names(&self) -> Result<&'a [String], InvocationError> {
        self.names
            .ok_or_else(|| InvocationError::Unfilled("names...".to_string()))
    }

    fn value(&self, name: &str) -> Result<&'a str, InvocationError> {
        let value = match name {
            "file" => self.file,
            "report" => self.report,
            _ => return Err(InvocationError::Unknown(name.to_string())),
        };
        value.ok_or_else(|| InvocationError::Unfilled(name.to_string()))
    }
}

fn fill(argument: &str, values: &Values) -> Result<String, InvocationError> {
    let mut filled = String::new();
    let mut rest = argument;
    while let Some(open) = rest.find('{') {
        filled.push_str(&rest[..open]);
        let after_brace = &rest[open + 1..];
        match after_brace.find('}').map(|close| &after_brace[..close]) {
            Some(name) if is_placeholder(name) => {
                filled.push_str(values.value(name)?);
                rest = &after_brace[name.len() + 1..];
            }
            _ => {
                filled.push('{');
                rest = after_brace;
            }
        }
    }
    filled.push_str(rest);
    Ok(filled)
}

fn is_placeholder(name: &str) -> bool {
    let word = name.strip_suffix("...").unwrap_or(name);
    !word.is_empty() && word.chars().all(|character| character.is_ascii_lowercase())
}

#[cfg(test)]
mod tests;
