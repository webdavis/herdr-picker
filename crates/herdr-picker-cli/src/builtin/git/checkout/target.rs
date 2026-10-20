use super::super::Repo;
use crate::command::{CommandError, failed};

// A local branch is checked out by name, a remote one as a local branch tracking it (or the
// local branch of that name when one exists), and a tag or commit as a detached HEAD.
pub(super) fn target(
    repo: &Repo,
    kind: &str,
    name: &str,
) -> Result<(Vec<String>, String, bool), CommandError> {
    let one = |word: &str| vec![word.to_string()];
    Ok(match kind {
        "local" => (one(name), name.to_string(), false),
        "remote" => {
            let local = name.split_once('/').map_or(name, |(_, local)| local);
            let exists = repo
                .git(&[
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{local}"),
                ])
                .is_ok();
            match exists {
                true => (one(local), local.to_string(), false),
                false => (
                    vec!["--track".to_string(), name.to_string()],
                    name.to_string(),
                    false,
                ),
            }
        }
        "tag" => (one(name), name.to_string(), true),
        "commit" => {
            let about = repo.git(&["log", "-1", "--format=%h%x09%s", name])?;
            let about = String::from_utf8_lossy(&about).trim_end().to_string();
            let (short, subject) = about.split_once('\t').unwrap_or((name, ""));
            (one(name), format!("{short} \"{subject}\""), true)
        }
        other => {
            return Err(failed(format!(
                "nothing of kind {other} can be checked out"
            )));
        }
    })
}
