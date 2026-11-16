use super::TestId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Nearest { file: String, line: u32 },
    File(String),
    Ids(Vec<TestId>),
    Failed,
    Last,
    Changed,
    All,
}
