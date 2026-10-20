use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json::Value;

// Reads a transcript only as far as its first prompt; `pick` sees each record and the context
// gathered so far.
pub(super) fn first_prompt(
    file: &Path,
    pick: fn(&Value, &mut String) -> Option<String>,
) -> Option<(String, String)> {
    if file.extension()? != "jsonl" {
        return None;
    }
    let mut context = String::new();
    BufReader::new(File::open(file).ok()?)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .find_map(|record| pick(&record, &mut context))
        .map(|prompt| (context, prompt))
}

pub(super) fn claude_prompt(record: &Value, cwd: &mut String) -> Option<String> {
    if record["type"] != "user" || record["isSidechain"] == true || record["isMeta"] == true {
        return None;
    }
    *cwd = record["cwd"].as_str()?.to_string();
    typed(record["message"]["content"].as_str()?)
}

// Codex opens with the session's folder and id, then the files it was handed, then the prompt.
pub(super) fn codex_prompt(record: &Value, meta: &mut String) -> Option<String> {
    let payload = &record["payload"];
    if record["type"] == "session_meta" {
        *meta = format!("{}\t{}", payload["cwd"].as_str()?, payload["id"].as_str()?);
        return None;
    }
    if record["type"] != "response_item" || payload["role"] != "user" {
        return None;
    }
    let texts = payload["content"]
        .as_array()?
        .iter()
        .filter_map(|part| part["text"].as_str());
    texts.filter_map(typed).next()
}

// What the person typed, not what the tool wrapped around it.
fn typed(text: &str) -> Option<String> {
    let text = text.trim();
    let wrapped = text.starts_with('<') || text.starts_with("# AGENTS.md instructions");
    (!text.is_empty() && !wrapped).then(|| text.to_string())
}
