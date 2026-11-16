use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct RexSettings {
    pub pass_keys_to: Vec<String>,
    pub app_name: String,
}

impl Default for RexSettings {
    fn default() -> Self {
        Self {
            pass_keys_to: vec!["nvim".to_string()],
            app_name: "Rex Beta".to_string(),
        }
    }
}
