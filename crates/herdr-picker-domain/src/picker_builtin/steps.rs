use super::{BUILTIN_PICKERS, Live};

// The built-in steps a picker's next action opens; no key runs them.
pub(super) const STEP_BUILTINS: [(&str, Live); 33] = [
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
    ("network-hosts", Live::Never),
    ("network-dns", Live::Never),
    ("dns-lookup", Live::Always),
    ("network-iperf", Live::Never),
    ("iperf-command", Live::Always),
    ("network-capture", Live::Never),
    ("capture-command", Live::Always),
    ("capture-folder", Live::Always),
    ("captures-in", Live::Never),
    ("data-file", Live::Never),
    ("data-copy", Live::Never),
    ("send-to", Live::Never),
    ("quick-answer", Live::Always),
    ("answer", Live::Never),
    ("contact-fields", Live::Never),
    ("contact-call", Live::Never),
    ("chats", Live::Never),
    ("message-history", Live::Never),
    ("message-links", Live::Never),
    ("message-attachments", Live::Never),
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

// Whether a picker's or step's command runs again as the user types.
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
