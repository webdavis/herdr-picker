use std::fmt;
use std::path::Path;

use herdr_picker_domain::{Action, Location};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnclosedQuote;

impl fmt::Display for UnclosedQuote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a quote is never closed")
    }
}

// What turning an action into a shell command needs from the shell's surroundings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shell<'a> {
    pub editor: &'a str,
    pub cwd: &'a str,
    pub opener: &'a str,
}

// Edit and open come back as commands to run, not run here: bash records them in its history,
// and the picker may be a popup that closes before an editor it started would.
pub fn instruction(action: &Action, shell: &Shell) -> Result<Option<String>, UnclosedQuote> {
    Ok(match action {
        Action::Insert(text) => Some(format!("insert\t{text}")),
        Action::Cd(path) => Some(format!("cd\t{path}")),
        Action::Run(command) => Some(format!("run\t{command}")),
        Action::Edit(locations) => Some(format!("run\t{}", edit_command(locations, shell)?)),
        Action::Open(target) => Some(format!("run\t{} {}", shell.opener, shell_word(target))),
        _ => None,
    })
}

fn edit_command(locations: &[Location], shell: &Shell) -> Result<String, UnclosedQuote> {
    let mut words = split_words(shell.editor)?;
    if let Some(line) = locations.first().and_then(|location| location.line) {
        words.push(format!("+{line}"));
    }
    let mut paths: Vec<String> = Vec::new();
    for location in locations {
        let path = relative(&location.path, Path::new(shell.cwd));
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    // The editor would read a leading - as an option and + as a command.
    words.extend(
        paths
            .into_iter()
            .map(|path| match path.starts_with(['-', '+']) {
                true => format!("./{path}"),
                false => path,
            }),
    );
    Ok(words
        .iter()
        .map(|word| shell_word(word))
        .collect::<Vec<_>>()
        .join(" "))
}

fn relative(path: &str, cwd: &Path) -> String {
    match Path::new(path).strip_prefix(cwd) {
        Ok(inside) if inside.as_os_str().is_empty() => ".".to_string(),
        Ok(inside) => inside.to_string_lossy().into_owned(),
        Err(_) => path.to_string(),
    }
}

pub fn shell_word(text: &str) -> String {
    let plain =
        |character: char| character.is_ascii_alphanumeric() || "@%+=:,./-_".contains(character);
    if !text.is_empty() && text.chars().all(plain) {
        text.to_string()
    } else {
        shell_quote(text)
    }
}

pub fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

pub fn split_words(text: &str) -> Result<Vec<String>, UnclosedQuote> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        match character {
            _ if character.is_whitespace() => words.extend(word.take()),
            '\'' => {
                let quoted = word.get_or_insert_default();
                loop {
                    match characters.next().ok_or(UnclosedQuote)? {
                        '\'' => break,
                        inner => quoted.push(inner),
                    }
                }
            }
            '"' => {
                let quoted = word.get_or_insert_default();
                loop {
                    match characters.next().ok_or(UnclosedQuote)? {
                        '"' => break,
                        '\\' => match characters.next().ok_or(UnclosedQuote)? {
                            escaped @ ('"' | '\\' | '$' | '`') => quoted.push(escaped),
                            other => quoted.extend(['\\', other]),
                        },
                        inner => quoted.push(inner),
                    }
                }
            }
            '\\' => word.get_or_insert_default().extend(characters.next()),
            _ => word.get_or_insert_default().push(character),
        }
    }
    words.extend(word);
    Ok(words)
}

#[cfg(test)]
mod tests;
