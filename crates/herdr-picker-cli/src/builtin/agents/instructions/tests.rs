use std::fs;

use super::*;
use crate::builtin::tests::Tree;

fn write(tree: &Tree, inside: &str, text: &str) {
    let path = tree.0.join(inside);
    fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
    fs::write(path, text).expect("a file");
}

#[test]
fn every_agent_file_for_the_repository_is_found_once_with_its_kind() {
    let repo = Tree::new(
        "instructions-repo",
        &[
            "AGENTS.md",
            "docs/CLAUDE.md",
            ".claude/skills/review/SKILL.md",
            ".claude/commands/ship.md",
            ".claude/agents/critic.md",
            ".mcp.json",
            "GLOSSARY.md",
            "notes/handoff-friday.md",
            ".audit/run.tsv",
        ],
    );
    let home = Tree::new(
        "instructions-home",
        &[".codex/AGENTS.md", ".agents/skills/tidy/SKILL.md"],
    );
    let name = repo
        .0
        .file_name()
        .expect("a name")
        .to_string_lossy()
        .into_owned();
    let project = files::claude_project(&home.0, &repo.0);
    write(
        &home,
        &format!(
            ".claude/projects/{}/memory/MEMORY.md",
            project.file_name().expect("a name").to_string_lossy()
        ),
        "m",
    );
    write(
        &home,
        ".claude/plans/about.md",
        &format!("Plan for {name}."),
    );
    write(&home, ".claude/plans/other.md", "Nothing here.");
    write(&home, "plugin/skills/lint/SKILL.md", "s");
    write(
        &home,
        ".claude/plugins/installed_plugins.json",
        &format!(
            r#"{{"plugins": {{"lint@market": [{{"scope": "user", "installPath": "{}"}}],
                "other@market": [{{"scope": "project", "projectPath": "/elsewhere", "installPath": "/nowhere"}}]}}}}"#,
            home.path("plugin")
        ),
    );

    let found = instruction_files(&repo.0, &repo.0, Some(&home.0), None);
    let shown: Vec<(String, &str)> = found
        .iter()
        .map(|(path, kind)| {
            let shown = path
                .strip_prefix(&repo.0)
                .map(|inside| inside.display().to_string())
                .unwrap_or_else(|_| {
                    format!("~/{}", path.strip_prefix(&home.0).expect("home").display())
                });
            (shown, *kind)
        })
        .collect();

    assert_eq!(
        shown,
        vec![
            ("AGENTS.md".to_string(), "instructions"),
            (".claude/skills/review/SKILL.md".to_string(), "instructions"),
            ("docs/CLAUDE.md".to_string(), "instructions"),
            ("GLOSSARY.md".to_string(), "notes"),
            ("notes/handoff-friday.md".to_string(), "handoff"),
            (".claude/commands/ship.md".to_string(), "command"),
            (".claude/agents/critic.md".to_string(), "subagent"),
            (".mcp.json".to_string(), "settings"),
            (".audit/run.tsv".to_string(), "pstack"),
            ("~/.codex/AGENTS.md".to_string(), "instructions"),
            (
                format!(
                    "~/.claude/projects/{}/memory/MEMORY.md",
                    project.file_name().expect("a name").to_string_lossy()
                ),
                "memory"
            ),
            ("~/.agents/skills/tidy/SKILL.md".to_string(), "skill"),
            ("~/plugin/skills/lint/SKILL.md".to_string(), "skill"),
            ("~/.claude/plans/about.md".to_string(), "plan"),
        ]
    );
}
