use pwe_domain::test_engine::Severity;

pub(super) fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
        Severity::Help => "help",
        Severity::Other => "other",
    }
}

pub(super) fn severity_of(word: &str) -> Severity {
    [
        Severity::Error,
        Severity::Warning,
        Severity::Note,
        Severity::Help,
    ]
    .into_iter()
    .find(|severity| severity_name(*severity) == word)
    .unwrap_or(Severity::Other)
}
