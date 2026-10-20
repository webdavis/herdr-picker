use std::path::{Path, PathBuf};

use herdr_picker_domain::{Action, Location, PickerRow, Preview};

use crate::builtin::Places;
use crate::builtin::git::Repo;
use crate::builtin::page::notice;
use crate::builtin::paint::dim;
use crate::builtin::rows::{in_home, shown};

mod files;

pub(super) use files::claude_project;
use files::{entries, mentioning, plugin_roots, repo_files, skills};

pub(super) const INSTRUCTION_NAMES: [&str; 3] = ["AGENTS.md", "CLAUDE.md", "GEMINI.md"];
const NOTES: [&str; 3] = ["CONTEXT.md", "GLOSSARY.md", "TASKS.md"];

// Every file that tells an agent how to work here, each once, with what kind of file it is.
pub(super) fn instructions(places: &Places) -> Vec<PickerRow> {
    let root = Repo::at(&places.cwd).map_or_else(|_| places.cwd.clone(), |repo| repo.root);
    let found = instruction_files(
        &places.cwd,
        &root,
        places.home.as_deref(),
        places.fd.as_deref(),
    );
    if found.is_empty() {
        return vec![notice(
            "No agent files here",
            "AGENTS.md, CLAUDE.md, skills and the rest show up here.",
        )];
    }
    found
        .into_iter()
        .map(|(path, kind)| {
            let full = path.to_string_lossy().into_owned();
            let label = match path.starts_with(&root) {
                true => shown(&path, &root),
                false => in_home(&path, places.home.as_deref()),
            };
            PickerRow {
                label: format!("{label}  {}", dim(kind)),
                preview: Some(Preview::Path(full.clone())),
                enter: Action::Edit(vec![Location {
                    path: full,
                    line: None,
                }]),
                keys: Vec::new(),
                group: None,
            }
        })
        .collect()
}

pub(super) fn instruction_files(
    cwd: &Path,
    root: &Path,
    home: Option<&Path>,
    fd: Option<&Path>,
) -> Vec<(PathBuf, &'static str)> {
    let mut found: Vec<(PathBuf, &'static str)> = Vec::new();
    let mut add = |paths: Vec<PathBuf>, kind: &'static str| {
        for path in paths {
            if path.is_file() && found.iter().all(|(seen, _)| *seen != path) {
                found.push((path, kind));
            }
        }
    };
    let named = |folder: &Path, names: &[&str]| -> Vec<PathBuf> {
        names.iter().map(|name| folder.join(name)).collect()
    };
    for folder in cwd.ancestors() {
        add(named(folder, &INSTRUCTION_NAMES), "instructions");
    }
    let (in_repo, handoffs) = repo_files(root, fd);
    add(in_repo, "instructions");
    add(named(root, &NOTES), "notes");
    add(handoffs, "handoff");
    add(entries(&root.join(".claude/skills"), "SKILL.md"), "skill");
    add(entries(&root.join(".agents/skills"), "SKILL.md"), "skill");
    add(skills(&root.join(".claude/commands")), "command");
    add(skills(&root.join(".claude/agents")), "subagent");
    add(
        [
            ".claude/settings.json",
            ".claude/settings.local.json",
            ".mcp.json",
            ".config/wt.toml",
        ]
        .map(|name| root.join(name))
        .to_vec(),
        "settings",
    );
    add(skills(&root.join(".codex")), "settings");
    add(skills(&root.join(".audit")), "pstack");
    let Some(home) = home else {
        return found;
    };
    let name = root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    add(
        vec![
            home.join(".codex/AGENTS.md"),
            home.join(".claude/CLAUDE.md"),
        ],
        "instructions",
    );
    add(skills(&claude_project(home, root).join("memory")), "memory");
    add(entries(&home.join(".claude/skills"), "SKILL.md"), "skill");
    add(entries(&home.join(".agents/skills"), "SKILL.md"), "skill");
    for plugin in plugin_roots(home, root) {
        add(entries(&plugin.join("skills"), "SKILL.md"), "skill");
        add(skills(&plugin.join("commands")), "command");
        add(skills(&plugin.join("agents")), "subagent");
    }
    add(skills(&home.join(".claude/commands")), "command");
    add(skills(&home.join(".codex/prompts")), "command");
    add(skills(&home.join(".claude/agents")), "subagent");
    add(skills(&home.join(".codex/agents")), "subagent");
    add(mentioning(&home.join(".claude/plans"), &name), "plan");
    add(vec![home.join(".claude/pstack-models.md")], "pstack");
    add(
        skills(&home.join(".claude/orchestrate").join(&name)),
        "pstack",
    );
    found
}

#[cfg(test)]
mod tests;
