#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerStep {
    pub command: Vec<String>,
    pub sources: Vec<String>,
    pub live: bool,
}
