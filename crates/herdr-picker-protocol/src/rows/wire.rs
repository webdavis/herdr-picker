use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(
    untagged,
    expecting = "an action: \"insert\", \"copy\", \"copy_stay\", \"reload\", \"none\", \"back\", or one {\"<action>\": ...}"
)]
pub(super) enum WireAction {
    Bare(Bare),
    Tagged(Tagged),
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub(super) enum Bare {
    Insert,
    Copy,
    CopyStay,
    Reload,
    None,
    Back,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Tagged {
    Insert(Words),
    Copy(Words),
    CopyStay(Words),
    Cd(String),
    Run(String),
    Edit(OneOrMany<WireLocation>),
    Open(String),
    Next(WireNext),
    Confirm(WireConfirm),
    Source(String),
    State(serde_json::Value),
}

#[derive(Deserialize, Serialize)]
#[serde(
    untagged,
    expecting = "a preview: its text, {\"path\": \"<file or folder>\"}, or {\"command\": [\"<program>\", ...]}"
)]
pub(super) enum WirePreview {
    Text(String),
    Path { path: String },
    Command { command: Vec<String> },
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub(super) enum Words {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub(super) enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireLocation {
    pub(super) path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) line: Option<u32>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireNext {
    pub(super) picker: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) state: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) query: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireConfirm {
    pub(super) question: String,
    pub(super) action: Box<WireAction>,
}
