use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use super::Repo;
use super::paint::bold;
use crate::command::CommandError;

// The previews git alone cannot draw: a change's staged and unstaged parts, a conflict's three
// sides, and a committed file colored for its language.
pub(crate) fn git_preview(arguments: &[&str]) -> Result<String, CommandError> {
    match arguments {
        ["diff", root, name] => diff(&repo(root), name),
        ["stages", root, name, base, ours, theirs] => {
            let repo = repo(root);
            let sides = [("Base", base), ("Ours", ours), ("Theirs", theirs)].map(|(side, blob)| {
                let text = match blob.bytes().all(|byte| byte == b'0') {
                    true => Ok("(stage absent)".to_string()),
                    false => committed(&repo, blob, name),
                };
                text.map(|text| format!("{}\n{}", bold(side), text.trim_end_matches('\n')))
            });
            Ok(sides
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .join("\n\n"))
        }
        ["blob", root, blob, name] => committed(&repo(root), blob, name),
        _ => Err(CommandError::Usage),
    }
}

fn repo(root: &str) -> Repo {
    Repo {
        root: Path::new(root).to_path_buf(),
    }
}

fn diff(repo: &Repo, name: &str) -> Result<String, CommandError> {
    let mut sections = Vec::new();
    for (heading, staged) in [("Staged", true), ("Unstaged", false)] {
        let mut arguments = vec!["diff", "--color=always", "--no-ext-diff", "--no-textconv"];
        arguments.extend(staged.then_some("--cached"));
        arguments.extend(["--", name]);
        let shown = String::from_utf8_lossy(&repo.git(&arguments)?).into_owned();
        if !shown.is_empty() {
            sections.push(format!("{}\n{shown}", bold(heading)));
        }
    }
    Ok(sections.join("\n"))
}

// bat colors it for the language its name says; without bat it is shown plain.
fn committed(repo: &Repo, blob: &str, name: &str) -> Result<String, CommandError> {
    let content = repo.git(&["cat-file", "blob", blob])?;
    if content.contains(&0) {
        return Ok("[Binary file]".to_string());
    }
    let colored = Command::new("bat")
        .args([
            "--color=always",
            "--style=numbers",
            "--paging=never",
            "--wrap=never",
        ])
        .arg(format!("--file-name={name}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut bat| {
            if let Some(mut input) = bat.stdin.take() {
                input.write_all(&content)?;
            }
            bat.wait_with_output()
        });
    let shown = match colored {
        Ok(ran) if ran.status.success() => ran.stdout,
        _ => content,
    };
    Ok(String::from_utf8_lossy(&shown).into_owned())
}

#[cfg(test)]
mod tests;
