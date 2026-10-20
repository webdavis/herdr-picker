use std::fmt;

use super::PaneKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Draft(String),
    Blocked,
    InputUnseen,
    UnknownAgent(String),
    NothingToOpen,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft(draft) => write!(
                formatter,
                "its input line holds your typing ({draft}), so nothing was sent"
            ),
            Self::Blocked => write!(formatter, "it waits on an answer, so nothing was sent"),
            Self::InputUnseen => write!(
                formatter,
                "its input line could not be read, so nothing was sent"
            ),
            Self::UnknownAgent(agent) => {
                write!(
                    formatter,
                    "herdr-picker does not know how to hand {agent} a result"
                )
            }
            Self::NothingToOpen => write!(formatter, "a new pane opens only a command or a folder"),
        }
    }
}

// herdr sends typed text even into a half-typed prompt, where it joins the owner's draft, and
// into a question menu, where it would answer it.
// shortcut: a dim placeholder in the input line reads as a draft and refuses; strip faint text once one shows up.
pub fn guard(kind: &PaneKind, status: &str, screen: &str) -> Result<(), Refusal> {
    let prompt = match kind {
        PaneKind::Claude => '❯',
        PaneKind::Codex => '›',
        PaneKind::Shell => return shell_guard(screen),
        PaneKind::Agent(_) => return Ok(()),
    };
    if status == "blocked" {
        return Err(Refusal::Blocked);
    }
    let input = screen
        .lines()
        .rev()
        .find_map(|line| line.trim_start().strip_prefix(prompt))
        .ok_or(Refusal::InputUnseen)?;
    match input.trim() {
        "" => Ok(()),
        draft => Err(Refusal::Draft(draft.to_string())),
    }
}

// Starship's prompt characters and the classic shell ones, each followed by a space or the end.
const SHELL_PROMPTS: [&str; 6] = ["❯", "❮", "$", "#", "%", ">"];

// The cursor sits on the last line holding text; what follows its first prompt character is typed,
// so a draft holding a prompt character of its own still reads as a draft.
// shortcut: a prompt ending in none of SHELL_PROMPTS reads as unseen and refuses; teach it that prompt.
fn shell_guard(screen: &str) -> Result<(), Refusal> {
    let line = screen
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or(Refusal::InputUnseen)?;
    let typed = SHELL_PROMPTS
        .iter()
        .filter_map(|prompt| {
            line.match_indices(prompt)
                .map(|(at, _)| &line[at + prompt.len()..])
                .find(|rest| rest.is_empty() || rest.starts_with(' '))
        })
        .max_by_key(|rest| rest.len())
        .ok_or(Refusal::InputUnseen)?;
    match typed.trim() {
        "" => Ok(()),
        draft => Err(Refusal::Draft(draft.to_string())),
    }
}
