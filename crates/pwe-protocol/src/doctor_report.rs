use pwe_domain::{CheckResult, DoctorCheck};
use serde_json::json;

fn result_word(result: CheckResult) -> &'static str {
    match result {
        CheckResult::Passed => "passed",
        CheckResult::Failed => "failed",
    }
}

pub fn doctor_report_json(checks: &[DoctorCheck]) -> String {
    let checks: Vec<_> = checks
        .iter()
        .map(|check| {
            json!({
                "name": check.name,
                "result": result_word(check.result),
                "finding": check.finding,
                "what-to-do": check.what_to_do,
            })
        })
        .collect();
    json!({ "checks": checks }).to_string()
}

pub fn doctor_report_words(checks: &[DoctorCheck]) -> String {
    checks
        .iter()
        .map(|check| {
            let what_to_do = check
                .what_to_do
                .as_ref()
                .map(|what_to_do| format!("        What to do: {what_to_do}\n"))
                .unwrap_or_default();
            format!(
                "{:<8}{}: {}\n{what_to_do}",
                result_word(check.result),
                check.name,
                check.finding
            )
        })
        .collect()
}
