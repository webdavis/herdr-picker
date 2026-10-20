use herdr_picker_domain::{Action, Key};
use serde_json::{Map, Value};

use crate::builtin::rows::key;

// The switches a content search keeps between keystrokes, each written only when it is not
// the default, so a step's state reads as what was changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Switches {
    pub(super) regex: bool,
    pub(super) untracked: bool,
    pub(super) hidden: bool,
    pub(super) sort: bool,
    pub(super) worktree: Option<String>,
}

impl Switches {
    pub(super) fn read(state: Option<&str>) -> Self {
        let state: Value = state
            .and_then(|state| serde_json::from_str(state).ok())
            .unwrap_or(Value::Null);
        let flag = |name: &str, default: bool| state[name].as_bool().unwrap_or(default);
        Self {
            regex: flag("regex", false),
            untracked: flag("untracked", false),
            hidden: flag("hidden", true),
            sort: flag("sort", false),
            worktree: state["worktree"].as_str().map(str::to_string),
        }
    }

    pub(super) fn state(&self) -> String {
        let mut written = Map::new();
        for (name, on, default) in [
            ("hidden", self.hidden, true),
            ("regex", self.regex, false),
            ("sort", self.sort, false),
            ("untracked", self.untracked, false),
        ] {
            if on != default {
                written.insert(name.to_string(), Value::Bool(on));
            }
        }
        if let Some(worktree) = &self.worktree {
            written.insert("worktree".to_string(), Value::String(worktree.clone()));
        }
        Value::Object(written).to_string()
    }

    pub(super) fn flipped(&self, change: impl Fn(&mut Self)) -> String {
        let mut flipped = self.clone();
        change(&mut flipped);
        flipped.state()
    }

    // Alt-X, Alt-., Alt-S and, in a repository, Alt-U: each shows the search again switched.
    pub(super) fn keys(&self, in_git: bool) -> Vec<(Key, Action)> {
        let mut flips = vec![
            ("alt-x", self.flipped(|on| on.regex = !on.regex)),
            ("alt-.", self.flipped(|on| on.hidden = !on.hidden)),
            ("alt-s", self.flipped(|on| on.sort = !on.sort)),
        ];
        if in_git {
            flips.push(("alt-u", self.flipped(|on| on.untracked = !on.untracked)));
        }
        flips
            .into_iter()
            .map(|(name, state)| (key(name), Action::State(state)))
            .collect()
    }
}
