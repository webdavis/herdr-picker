// A script only a row's next opens, from the picker that declares it or a step after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerStep {
    // The program, then its own arguments.
    pub command: Vec<String>,
    // The first is the one the step opens on.
    pub sources: Vec<String>,
    // Its rows follow the query, so its command runs again as the user types.
    pub live: bool,
}
