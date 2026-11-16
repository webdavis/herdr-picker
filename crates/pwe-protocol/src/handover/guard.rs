use std::fmt;

use super::PaneKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Draft(String),
    Blocked,
    InputUnseen,
    UnknownAgent(String),
    NothingToOpen,
    OnlyInHerdr,
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
                write!(formatter, "pwe does not know how to hand {agent} a result")
            }
            Self::NothingToOpen => write!(formatter, "a new pane opens only a command or a folder"),
            Self::OnlyInHerdr => write!(formatter, "only herdr can do that, so nothing was sent"),
        }
    }
}

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

const STARSHIP_AND_CLASSIC_PROMPT_CHARACTERS: [char; 6] = ['❯', '❮', '$', '#', '%', '>'];

fn shell_guard(screen: &str) -> Result<(), Refusal> {
    let line = screen
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or(Refusal::InputUnseen)?;
    let (before, typed) = line
        .char_indices()
        .filter(|(_, character)| STARSHIP_AND_CLASSIC_PROMPT_CHARACTERS.contains(character))
        .map(|(at, character)| (&line[..at], &line[at + character.len_utf8()..]))
        .find(|(_, rest)| rest.is_empty() || rest.starts_with(' '))
        .ok_or(Refusal::InputUnseen)?;
    if !before.split_whitespace().all(prompt_like) {
        return Err(Refusal::InputUnseen);
    }
    match typed.trim() {
        "" => Ok(()),
        draft => Err(Refusal::Draft(draft.to_string())),
    }
}

fn prompt_like(word: &str) -> bool {
    let wrapped = |open, close| word.starts_with(open) && word.ends_with(close);
    word.contains('@')
        || word.starts_with(['~', '/'])
        || wrapped('(', ')')
        || wrapped('[', ']')
        || word.chars().all(|character| {
            !character.is_alphanumeric()
                && !STARSHIP_AND_CLASSIC_PROMPT_CHARACTERS.contains(&character)
        })
}
