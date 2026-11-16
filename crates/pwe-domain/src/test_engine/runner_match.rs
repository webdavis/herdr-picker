use super::RunnerSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedRunner {
    pub spec: RunnerSpec,
    pub matched_by: String,
    pub command_prefix: Vec<String>,
}

const MISE_FILE: &str = "mise.toml";
const MISE_PREFIX: [&str; 3] = ["mise", "exec", "--"];

pub fn matched_runners(specs: &[RunnerSpec], present: impl Fn(&str) -> bool) -> Vec<MatchedRunner> {
    let command_prefix: Vec<String> = match present(MISE_FILE) {
        true => MISE_PREFIX.map(String::from).to_vec(),
        false => Vec::new(),
    };
    specs
        .iter()
        .filter_map(|spec| {
            let matched_by = spec.detect_files.iter().find(|file| present(file))?;
            Some(MatchedRunner {
                spec: spec.clone(),
                matched_by: matched_by.clone(),
                command_prefix: command_prefix.clone(),
            })
        })
        .collect()
}
