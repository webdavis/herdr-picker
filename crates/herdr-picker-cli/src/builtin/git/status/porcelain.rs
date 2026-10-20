use super::super::{Repo, strings};
use crate::command::CommandError;

// The states a file can be in, in the order the status picker lists them: what needs you
// first, then what git status shows, then the quiet tracked files.
pub(super) const STATES: [&str; 6] = [
    "conflicts",
    "staged",
    "unstaged",
    "untracked",
    "stashes",
    "tracked",
];

pub(super) struct File {
    pub(super) name: String,
    // Never empty; the first is the one it is listed under.
    pub(super) states: Vec<&'static str>,
    // A conflict's base, ours and theirs blobs.
    pub(super) stages: Option<[String; 3]>,
}

impl File {
    pub(super) fn changed(&self) -> bool {
        self.states
            .iter()
            .any(|state| matches!(*state, "staged" | "unstaged"))
    }

    pub(super) fn stashable(&self) -> bool {
        self.changed() || self.states.contains(&"untracked")
    }
}

#[derive(Default)]
pub(super) struct Porcelain {
    pub(super) head: String,
    pub(super) upstream: String,
    pub(super) ahead: String,
    pub(super) behind: String,
    pub(super) files: Vec<File>,
}

impl Porcelain {
    // One read-only git status: it takes no lock, so git running elsewhere in the repository
    // at the same moment does not fail it.
    pub(super) fn read(repo: &Repo) -> Result<Self, CommandError> {
        let records = strings(&repo.git(&[
            "--no-optional-locks",
            "status",
            "--porcelain=v2",
            "-z",
            "--branch",
            "--no-renames",
            "--untracked-files=all",
        ])?);
        let mut status = Self::default();
        for record in records {
            status.take(&record);
        }
        for name in strings(&repo.git(&["ls-files", "-z", "--cached"])?) {
            match status.files.iter_mut().find(|file| file.name == name) {
                Some(file) => file.states.push("tracked"),
                None => status.files.push(File {
                    name,
                    states: vec!["tracked"],
                    stages: None,
                }),
            }
        }
        let order = |file: &File| STATES.iter().position(|state| *state == file.states[0]);
        status
            .files
            .retain(|file| !file.name.split('/').any(|part| part == ".git"));
        status.files.sort_by_key(order);
        Ok(status)
    }

    fn take(&mut self, record: &str) {
        let (kind, rest) = record.split_once(' ').unwrap_or((record, ""));
        let header = |name: &str| rest.strip_prefix(name).map(str::to_string);
        match kind {
            "#" => {
                if let Some(head) = header("branch.head ") {
                    self.head = head;
                } else if let Some(upstream) = header("branch.upstream ") {
                    self.upstream = upstream;
                } else if let Some(counts) = header("branch.ab ") {
                    let (ahead, behind) = counts.split_once(' ').unwrap_or_default();
                    self.ahead = ahead.trim_start_matches('+').to_string();
                    self.behind = behind.trim_start_matches('-').to_string();
                }
            }
            "1" => {
                let fields: Vec<&str> = rest.splitn(8, ' ').collect();
                let xy = fields[0].as_bytes();
                let states = [("staged", xy[0]), ("unstaged", xy[1])]
                    .into_iter()
                    .filter(|(_, flag)| *flag != b'.')
                    .map(|(state, _)| state)
                    .collect();
                self.add(fields[7], states, None);
            }
            "u" => {
                let fields: Vec<&str> = rest.splitn(10, ' ').collect();
                let blob = |index: usize| fields[index].to_string();
                self.add(
                    fields[9],
                    vec!["conflicts"],
                    Some([blob(6), blob(7), blob(8)]),
                );
            }
            "?" => self.add(rest, vec!["untracked"], None),
            _ => {}
        }
    }

    fn add(&mut self, name: &str, states: Vec<&'static str>, stages: Option<[String; 3]>) {
        self.files.push(File {
            name: name.to_string(),
            states,
            stages,
        });
    }
}
