use super::Live;

// The built-in steps a picker's next action opens; no key runs them.
pub(super) const STEP_BUILTINS: [(&str, Live); 13] = [
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
];

pub fn is_step_builtin(name: &str) -> bool {
    STEP_BUILTINS.iter().any(|(step, _)| *step == name)
}
