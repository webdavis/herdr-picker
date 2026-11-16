use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::inputs::validate;

const SHOWN: [&str; 4] = ["hostname", "user", "port", "proxyjump"];

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SshHost {
    pub(super) alias: String,
    pub(super) file: PathBuf,
    pub(super) settings: Vec<String>,
}

pub(super) fn ssh_hosts(config: &Path, home: Option<&Path>) -> Vec<SshHost> {
    let base = config.parent().unwrap_or(Path::new("/"));
    let mut found = Found {
        base,
        home,
        seen: HashSet::new(),
        hosts: Vec::new(),
    };
    found.read(config);
    found.hosts
}

struct Found<'a> {
    base: &'a Path,
    home: Option<&'a Path>,
    seen: HashSet<PathBuf>,
    hosts: Vec<SshHost>,
}

impl Found<'_> {
    fn read(&mut self, path: &Path) {
        let Ok(text) = fs::read_to_string(path) else {
            return;
        };
        if !self
            .seen
            .insert(path.canonicalize().unwrap_or(path.to_path_buf()))
        {
            return;
        }
        let mut block = self.hosts.len()..self.hosts.len();
        for (keyword, values) in text
            .lines()
            .filter_map(words_split_on_spaces_without_escapes)
        {
            match keyword.as_str() {
                "include" => {
                    for pattern in &values {
                        expanded_with_wildcards_in_the_last_part_only(&self.included(pattern))
                            .iter()
                            .for_each(|included| self.read(included));
                    }
                    block = self.hosts.len()..self.hosts.len();
                }
                "host" => {
                    let start = self.hosts.len();
                    let fixed = values.iter().filter(|alias| fixed(alias));
                    self.hosts.extend(fixed.map(|alias| SshHost {
                        alias: alias.clone(),
                        file: path.to_path_buf(),
                        settings: Vec::new(),
                    }));
                    block = start..self.hosts.len();
                }
                "match" => block = self.hosts.len()..self.hosts.len(),
                shown if SHOWN.contains(&shown) => {
                    let name = format!("{}{}", shown[..1].to_uppercase(), &shown[1..]);
                    let setting = format!("{name}: {}", values.join(" "));
                    self.hosts[block.clone()]
                        .iter_mut()
                        .for_each(|host| host.settings.push(setting.clone()));
                }
                _ => {}
            }
        }
    }

    fn included(&self, pattern: &str) -> PathBuf {
        match (pattern.strip_prefix("~/"), self.home) {
            (Some(inside), Some(home)) => home.join(inside),
            _ => self.base.join(pattern),
        }
    }
}

fn words_split_on_spaces_without_escapes(line: &str) -> Option<(String, Vec<String>)> {
    let mut words: Vec<String> = line
        .split_whitespace()
        .take_while(|word| !word.starts_with('#'))
        .map(|word| word.trim_matches('"').to_string())
        .collect();
    if words.is_empty() {
        return None;
    }
    let first = words.remove(0);
    let (keyword, value) = first.split_once('=').unwrap_or((&first, ""));
    if !value.is_empty() {
        words.insert(0, value.to_string());
    }
    Some((keyword.to_lowercase(), words))
}

fn fixed(alias: &str) -> bool {
    !alias.contains(['*', '?', '!']) && validate("host", alias).is_ok()
}

fn expanded_with_wildcards_in_the_last_part_only(pattern: &Path) -> Vec<PathBuf> {
    let name = pattern
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    let (Some(folder), Some(name)) = (pattern.parent(), name) else {
        return Vec::new();
    };
    if !name.contains(['*', '?']) {
        return vec![pattern.to_path_buf()];
    }
    let mut found: Vec<PathBuf> = fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| matches(&name, &entry.file_name().to_string_lossy()))
        .map(|entry| entry.path())
        .collect();
    found.sort();
    found
}

fn matches(pattern: &str, name: &str) -> bool {
    let (mut wanted, mut named) = (pattern.chars(), name.chars());
    match (wanted.next(), named.next()) {
        (None, None) => true,
        (Some('*'), next) => {
            matches(wanted.as_str(), name) || (next.is_some() && matches(pattern, named.as_str()))
        }
        (Some(want), Some(have)) if want == '?' || want == have => {
            matches(wanted.as_str(), named.as_str())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
