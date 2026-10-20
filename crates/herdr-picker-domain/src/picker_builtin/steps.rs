use super::Live;

// The built-in steps a picker's next action opens; no key runs them.
pub(super) const STEP_BUILTINS: [(&str, Live); 15] = [
    ("commit-files", Live::Never),
    ("checkout", Live::Never),
    ("stash-files", Live::Always),
    ("rename-branch", Live::Always),
    ("pull", Live::Never),
    ("prune", Live::Never),
    ("change-setting", Live::Always),
    ("unified-logs", Live::Never),
    ("crashes", Live::Never),
    ("network-scan", Live::Always),
    ("capture-analysis", Live::Never),
    ("capture-filter", Live::Always),
    ("network-command", Live::Always),
    ("data-file", Live::Never),
    ("data-copy", Live::Never),
];

// The step builtins that read more than one source; the first is the one a step opens on.
const STEP_SOURCES: [(&str, &[&str]); 1] = [(
    "data-file",
    &["paths", "zoom", "keys", "levels", "shape", "tables"],
)];

pub fn is_step_builtin(name: &str) -> bool {
    STEP_BUILTINS.iter().any(|(step, _)| *step == name)
}

pub fn step_sources(name: &str) -> &'static [&'static str] {
    STEP_SOURCES
        .iter()
        .find(|(step, _)| *step == name)
        .map_or(&[], |(_, sources)| sources)
}
