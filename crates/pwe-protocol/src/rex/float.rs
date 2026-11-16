use pwe_domain::FLOAT_LABEL;
use serde_json::json;

use super::KeyFacts;

const TERMINAL_FLAVOR: &str = "com.superlogical.terminal.shell";
const CENTRED_FLOAT: [f64; 4] = [0.1, 0.1, 0.8, 0.8];

pub fn float_request(facts: &KeyFacts, command: &[String]) -> String {
    json!({
        "session_id": facts.session,
        "focus": true,
        "bounds": CENTRED_FLOAT,
        "layout": {"block": {
            "flavor": TERMINAL_FLAVOR,
            "label": FLOAT_LABEL,
            "options": {"shell": "none", "cwd": facts.folder, "command": command},
        }},
    })
    .to_string()
}
