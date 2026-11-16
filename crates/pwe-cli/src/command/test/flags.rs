use pwe_protocol::test_engine::{Hint, TestError, TestErrorCode};

#[derive(Debug, Default)]
pub(super) struct Flags {
    pub(super) format: Option<String>,
    pub(super) all: bool,
    pub(super) file: Option<String>,
    pub(super) ids: Vec<String>,
    pub(super) runner: Option<String>,
}

pub(super) fn read_flags(words: &[&str], accepted: &[&str]) -> Result<Flags, TestError> {
    let mut flags = Flags::default();
    let mut rest = words.iter().copied().peekable();
    while let Some(flag) = rest.next() {
        if !accepted.contains(&flag) {
            return Err(unknown_flag(flag, accepted));
        }
        let mut value = || {
            rest.next()
                .map(str::to_string)
                .ok_or_else(|| missing_value(flag, accepted))
        };
        match flag {
            "--format" => flags.format = Some(value()?),
            "--file" => flags.file = Some(value()?),
            "--runner" => flags.runner = Some(value()?),
            "--all" => flags.all = true,
            _ => {
                while let Some(id) = rest.next_if(|word| !word.starts_with("--")) {
                    flags.ids.push(id.to_string());
                }
                if flags.ids.is_empty() {
                    return Err(missing_value(flag, accepted));
                }
            }
        }
    }
    Ok(flags)
}

fn unknown_flag(flag: &str, accepted: &[&str]) -> TestError {
    TestError::new(
        TestErrorCode::BadFlag,
        format!("pwe test does not know the flag `{flag}`."),
    )
    .with_hint(accepted_flags(accepted))
}

fn missing_value(flag: &str, accepted: &[&str]) -> TestError {
    TestError::new(
        TestErrorCode::BadFlag,
        format!("The flag `{flag}` needs a value after it."),
    )
    .with_hint(accepted_flags(accepted))
}

fn accepted_flags(accepted: &[&str]) -> Hint {
    Hint {
        accepted: accepted.iter().map(|flag| flag.to_string()).collect(),
        ..Hint::default()
    }
}
