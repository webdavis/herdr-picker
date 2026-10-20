use std::path::Path;

use herdr_picker_domain::NewPane;

use crate::shell::{shell_word, split_words};

mod guard;
mod instruction;
mod terminal;

pub use guard::{Refusal, guard};
pub use instruction::{Instruction, Verb, parse_instruction};
use terminal::needs_terminal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneKind {
    Shell,
    Claude,
    Codex,
    Agent(String),
}

impl PaneKind {
    pub fn of(agent: Option<&str>) -> Self {
        match agent {
            None => Self::Shell,
            Some("claude") => Self::Claude,
            Some("codex") => Self::Codex,
            Some(other) => Self::Agent(other.to_string()),
        }
    }
}

// What the pane under a picker opened from herdr runs, and how to tell its words apart.
pub struct Underneath<'a> {
    pub kind: PaneKind,
    pub cwd: &'a str,
    pub editor: &'a str,
    pub is_path: &'a dyn Fn(&str) -> bool,
    pub is_program: &'a dyn Fn(&str) -> bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handover {
    // Typed into the pane underneath, then Enter when `submit`.
    Type {
        text: String,
        submit: bool,
    },
    // A new pane in `folder`, or the folder underneath, running `run` if any.
    Open {
        pane: NewPane,
        folder: Option<String>,
        run: Option<String>,
    },
}

pub fn handover(instruction: &Instruction, underneath: &Underneath) -> Result<Handover, Refusal> {
    let text = instruction.text.clone();
    if let Some(pane) = instruction.pane {
        return match instruction.verb {
            Verb::Insert => Err(Refusal::NothingToOpen),
            Verb::Cd => Ok(Handover::Open {
                pane,
                folder: Some(absolute(&text, underneath.cwd)),
                run: None,
            }),
            Verb::Run => Ok(Handover::Open {
                pane,
                folder: None,
                run: Some(text),
            }),
        };
    }
    let agent = match &underneath.kind {
        PaneKind::Shell => return Ok(into_shell(instruction.verb, text)),
        PaneKind::Agent(name) => return Err(Refusal::UnknownAgent(name.clone())),
        agent => agent,
    };
    let typed = |text: String, submit| Ok(Handover::Type { text, submit });
    match instruction.verb {
        Verb::Insert => typed(inserted(agent, text, underneath), false),
        Verb::Cd => typed(
            format!("! cd {}", shell_word(&absolute(&text, underneath.cwd))),
            true,
        ),
        Verb::Run if needs_terminal(&text, underneath.editor) => Ok(Handover::Open {
            pane: NewPane::Right,
            folder: None,
            run: Some(text),
        }),
        Verb::Run => typed(format!("! {text}"), true),
    }
}

fn into_shell(verb: Verb, text: String) -> Handover {
    let (text, submit) = match verb {
        Verb::Insert => (text, false),
        Verb::Cd => (format!("cd {}", shell_word(&text)), true),
        Verb::Run => (text, true),
    };
    Handover::Type { text, submit }
}

fn inserted(agent: &PaneKind, text: String, underneath: &Underneath) -> String {
    let words = split_words(&text).unwrap_or_default();
    let all_paths = !words.is_empty() && words.iter().all(|word| (underneath.is_path)(word));
    let command = words.len() > 1 && (underneath.is_program)(&words[0]);
    match (all_paths, agent) {
        (true, PaneKind::Claude) => words
            .iter()
            .map(|path| format!("@{path}"))
            .collect::<Vec<_>>()
            .join(" "),
        (false, _) if command => format!("! {text}"),
        _ => text,
    }
}

fn absolute(path: &str, cwd: &str) -> String {
    Path::new(cwd).join(path).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests;
