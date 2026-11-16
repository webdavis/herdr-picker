use std::path::Path;
use std::process::Command;
use std::thread;

use super::super::{output, strings};
use super::state::Switches;
use crate::command::{CommandError, failed};

const CHUNK: usize = 512;

pub(super) struct Hit {
    pub(super) name: String,
    pub(super) line: u32,
    pub(super) text: String,
}

// The files a search in a repository reads: the tracked ones, or the untracked ones.
pub(super) fn names(root: &Path, switches: &Switches) -> Result<Vec<String>, CommandError> {
    let listing: &[&str] = match switches.untracked {
        true => &["ls-files", "-z", "--others", "--exclude-standard"],
        false => &["ls-files", "-z", "--cached"],
    };
    let found = output(Command::new("git").arg("-C").arg(root).args(listing))?;
    Ok(strings(&found)
        .into_iter()
        .filter(|name| !name.split('/').any(|part| part == ".git"))
        .filter(|name| switches.hidden || !name.split('/').any(|part| part.starts_with('.')))
        .filter(|name| root.join(name).is_file())
        .collect())
}

// The listed files are searched a chunk at a time, every chunk at once; the hits come back in
// the order of the list, then by line, since ripgrep reports a chunk in whatever order its
// threads finish.
pub(super) fn search(
    root: &Path,
    names: Option<&[String]>,
    query: &str,
    switches: &Switches,
) -> Result<Vec<Hit>, CommandError> {
    let Some(names) = names else {
        let mut hits = ripgrep(root, &[".".to_string()], query, switches)?;
        hits.sort_by(|one, other| (&one.name, one.line).cmp(&(&other.name, other.line)));
        return Ok(hits);
    };
    let chunks: Vec<Result<Vec<Hit>, CommandError>> = thread::scope(|scope| {
        let running: Vec<_> = names
            .chunks(CHUNK)
            .map(|chunk| scope.spawn(move || ripgrep(root, chunk, query, switches)))
            .collect();
        running
            .into_iter()
            .map(|chunk| {
                chunk
                    .join()
                    .unwrap_or_else(|_| Err(failed("a search stopped")))
            })
            .collect()
    });
    let mut hits = Vec::new();
    for chunk in chunks {
        hits.extend(chunk?);
    }
    let position = |name: &str| names.iter().position(|listed| listed == name);
    hits.sort_by_key(|hit| (position(&hit.name), hit.line));
    Ok(hits)
}

fn ripgrep(
    root: &Path,
    files: &[String],
    query: &str,
    switches: &Switches,
) -> Result<Vec<Hit>, CommandError> {
    let mut command = Command::new("rg");
    command.current_dir(root).args([
        "--null",
        "--line-number",
        "--no-heading",
        "--with-filename",
        "--color=never",
        "--smart-case",
        "--no-ignore",
    ]);
    if switches.hidden {
        command.arg("--hidden");
    }
    if !switches.regex {
        command.arg("--fixed-strings");
    }
    command.arg("--").arg(query).args(files);
    let ran = command
        .output()
        .map_err(|failure| failed(format!("cannot run rg: {failure}")))?;
    if !matches!(ran.status.code(), Some(0 | 1)) {
        return Err(failed(String::from_utf8_lossy(&ran.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&ran.stdout)
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once('\0')?;
            let (number, text) = rest.split_once(':')?;
            Some(Hit {
                name: name.strip_prefix("./").unwrap_or(name).to_string(),
                line: number.parse().ok()?,
                text: text
                    .chars()
                    .map(|c| if c.is_control() { ' ' } else { c })
                    .collect(),
            })
        })
        .collect())
}
