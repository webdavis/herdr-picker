use super::{BuiltinPicker, Live};

// Herdr's agents, and the handoff sources kept from the agents picker the shell suite had.
pub(super) const AGENTS: BuiltinPicker = BuiltinPicker {
    name: "agents",
    key: "ctrl-t a",
    description: "Agents and tabs in herdr, the files that instruct agents here, saved failures, and past sessions",
    sources: &[
        "everything",
        "needs you",
        "done",
        "agents",
        "terminals",
        "workspaces",
        "recent",
        "worktrees",
        "instructions",
        "failures",
        "start",
        "sessions",
    ],
    live: Live::Never,
};
