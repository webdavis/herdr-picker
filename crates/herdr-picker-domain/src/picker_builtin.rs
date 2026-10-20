mod agents;
mod pickers;
mod steps;

use agents::AGENTS;
use pickers::PICKERS;
use steps::STEP_BUILTINS;
pub use steps::{is_step_builtin, step_sources};

// The pickers herdr-picker ships, on their keys unless the config turns them off or moves them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinPicker {
    pub name: &'static str,
    pub key: &'static str,
    pub description: &'static str,
    // The first is the one a picker opens on.
    pub sources: &'static [&'static str],
    pub live: Live,
}

// Whether a picker's rows follow the query, so its command runs again as the user types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Live {
    Never,
    Always,
    On(&'static [&'static str]),
}

pub const BUILTIN_PICKERS: [BuiltinPicker; 15] = [
    BuiltinPicker {
        name: "files",
        key: "ctrl-t f",
        description: "Files here, hidden ones, at home, on the whole disk, or used lately",
        sources: &["this folder", "hidden", "home", "whole disk", "recent"],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "folders",
        key: "ctrl-t d",
        description: "Folders to change into",
        sources: &["subfolders", "parents", "git", "bookmarks"],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "bookmark",
        key: "ctrl-t ctrl-b",
        description: "Bookmark this folder for the folders picker",
        sources: &[],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "status",
        key: "ctrl-g s",
        description: "Every file Git has an opinion about, and the stashes",
        sources: &[
            "all",
            "conflicts",
            "staged",
            "unstaged",
            "untracked",
            "stashes",
            "tracked",
            "ignored",
        ],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "history",
        key: "ctrl-g l",
        description: "Commits, the reflog, or the commits whose changes match what is typed",
        sources: &["commits", "reflog", "changes"],
        live: Live::On(&["changes"]),
    },
    BuiltinPicker {
        name: "branches",
        key: "ctrl-g b",
        description: "Branches, tags and worktrees, or the repos with something to prune",
        sources: &["branches", "repos"],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "contents",
        key: "ctrl-g /",
        description: "Lines matching what is typed",
        sources: &[],
        live: Live::Always,
    },
    BuiltinPicker {
        name: "git-config",
        key: "ctrl-g v",
        description: "Git settings, where each comes from, and which value wins",
        sources: &[],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "recipes",
        key: "ctrl-t j",
        description: "Recipes in this project's justfile",
        sources: &[],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "tests",
        key: "ctrl-t t",
        description: "Tests in this project, compiler and linter errors, or the saved quickfix list",
        sources: &["tests", "errors", "quickfix"],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "processes",
        key: "ctrl-t p",
        description: "Processes and the sockets they hold, services, or the last hour of logs",
        sources: &["processes", "services", "logs"],
        live: Live::Never,
    },
    BuiltinPicker {
        name: "network",
        key: "ctrl-t n",
        description: "Devices on the network, saved captures, checks of a URL, or network commands",
        sources: &["devices", "captures", "web checks", "commands"],
        live: Live::On(&["web checks"]),
    },
    BuiltinPicker {
        name: "data",
        key: "ctrl-t i",
        description: "JSON, YAML, TOON and SQLite files here, or the clipboard, by path and value",
        sources: &["files", "clipboard", "temporal"],
        live: Live::Never,
    },
    AGENTS,
    PICKERS,
];

pub fn follows_query(builtin: &str, source: Option<&str>) -> bool {
    let shipped = BUILTIN_PICKERS
        .iter()
        .find(|picker| picker.name == builtin)
        .map(|picker| picker.live);
    let step = STEP_BUILTINS
        .iter()
        .find(|(step, _)| *step == builtin)
        .map(|(_, live)| *live);
    match shipped.or(step) {
        Some(Live::Always) => true,
        Some(Live::On(sources)) => source.is_some_and(|source| sources.contains(&source)),
        Some(Live::Never) | None => false,
    }
}
