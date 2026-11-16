#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub result: CheckResult,
    pub finding: String,
    pub what_to_do: Option<String>,
}

impl DoctorCheck {
    pub fn passed(name: &str, finding: String) -> Self {
        Self {
            name: name.to_string(),
            result: CheckResult::Passed,
            finding,
            what_to_do: None,
        }
    }

    pub fn failed(name: &str, finding: String, what_to_do: String) -> Self {
        Self {
            name: name.to_string(),
            result: CheckResult::Failed,
            finding,
            what_to_do: Some(what_to_do),
        }
    }
}
