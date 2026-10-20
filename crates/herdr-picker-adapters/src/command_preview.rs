use std::process::{Command, Stdio};

// What the program prints; one that fails shows what it said on stderr too.
pub fn command_preview(argv: &[String]) -> String {
    let Some((program, arguments)) = argv.split_first() else {
        return String::new();
    };
    match Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
    {
        Ok(ran) if ran.status.success() => String::from_utf8_lossy(&ran.stdout).into_owned(),
        Ok(ran) => {
            String::from_utf8_lossy(&ran.stdout).into_owned()
                + &String::from_utf8_lossy(&ran.stderr)
        }
        Err(failure) => format!("Cannot run {program}: {failure}"),
    }
}

#[cfg(test)]
mod tests;
