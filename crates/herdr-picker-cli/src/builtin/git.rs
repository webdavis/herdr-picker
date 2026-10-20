use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use herdr_picker_protocol::shell_word;

use super::rows::RowWriter;
use super::{Asked, Places};
use crate::command::{CommandError, failed};

mod branches;
mod checkout;
mod contents;
mod history;
mod inputs;
use super::paint;
mod preview;
mod prune;
mod settings;
mod status;
mod steps;
mod worktrunk;

pub(crate) use preview::git_preview;

// The repository a git picker reads, by its top folder.
pub(super) struct Repo {
    pub(super) root: PathBuf,
}

impl Repo {
    pub(super) fn at(folder: &Path) -> Result<Self, CommandError> {
        let top = output(
            Command::new("git")
                .arg("-C")
                .arg(folder)
                .args(["rev-parse", "--show-toplevel"]),
        )
        .map_err(|_| failed("This picker needs a Git repository."))?;
        Ok(Self {
            root: PathBuf::from(OsStr::from_bytes(top.trim_ascii_end())),
        })
    }

    pub(super) fn git(&self, arguments: &[&str]) -> Result<Vec<u8>, CommandError> {
        output(&mut self.command(arguments))
    }

    pub(super) fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new("git");
        command
            .args(["--no-pager", "-c", "core.quotePath=false", "-C"])
            .arg(&self.root)
            .args(arguments);
        command
    }

    // A command for the shell to run in this repository, wherever the shell is.
    pub(super) fn shell(&self, arguments: &[&str]) -> String {
        let root = self.root.to_string_lossy();
        ["git", "-C", &root]
            .iter()
            .chain(arguments)
            .map(|word| shell_word(word))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(super) fn output(command: &mut Command) -> Result<Vec<u8>, CommandError> {
    let ran = command.stdin(Stdio::null()).output().map_err(failed)?;
    if ran.status.success() {
        return Ok(ran.stdout);
    }
    let said = String::from_utf8_lossy(&ran.stderr).trim().to_string();
    Err(failed(if said.is_empty() {
        format!("{:?} stopped with {}", command.get_program(), ran.status)
    } else {
        said
    }))
}

// Git's -z output: names and fields split at NUL.
pub(super) fn strings(data: &[u8]) -> Vec<String> {
    data.split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect()
}

// A preview that runs herdr-picker itself, for the ones git alone cannot draw.
pub(super) fn own_preview(arguments: &[&str]) -> Vec<String> {
    let program = env::current_exe().map_or_else(
        |_| "herdr-picker".into(),
        |path| path.to_string_lossy().into_owned(),
    );
    std::iter::once(program)
        .chain(std::iter::once("git-preview".to_string()))
        .chain(arguments.iter().map(|word| word.to_string()))
        .collect()
}

// A worktrunk command for the shell to run on a repository, wherever the shell is.
pub(super) fn wt(root: &Path, arguments: &[&str]) -> String {
    let root = root.to_string_lossy();
    ["wt", "-C", &root]
        .iter()
        .chain(arguments)
        .map(|word| shell_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
}

// The git pickers and their steps, or None for a name that is not one of them.
pub(super) fn git_rows(
    name: &str,
    arguments: &[&str],
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Option<Result<(), CommandError>> {
    Some(match (name, arguments) {
        ("status", []) => status::status(asked.source.unwrap_or("all"), places, rows),
        ("history", []) => history::history(asked, places, rows),
        ("commit-files", [commit]) => history::commit_files(commit, places, rows),
        ("checkout", [kind, target]) => checkout::checkout(kind, target, places, rows),
        ("branches", []) => branches::branches(asked.source.unwrap_or("branches"), places, rows),
        ("pull", []) => branches::pull(places, rows),
        ("prune", roots) => prune::prune(roots, rows),
        ("contents", []) => contents::contents(asked, places, rows),
        ("git-config", []) => settings::settings(places, rows),
        ("stash-files", names) => inputs::stash_files(names, asked.query, places, rows),
        ("rename-branch", [branch]) => inputs::rename_branch(branch, asked.query, places, rows),
        ("change-setting", [file, name, scope, shown]) => {
            inputs::change_setting([file, name, scope, shown], asked.query, rows)
        }
        _ => return None,
    })
}

#[cfg(test)]
pub(crate) mod fixture;
