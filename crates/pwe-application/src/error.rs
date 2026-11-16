use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpError {
    Unreachable(String),
    Transport(String),
    Malformed(String),
    Server { code: String, message: String },
    BothPathsFailed { cli: String, socket: String },
}

impl fmt::Display for JumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(detail) => write!(f, "unreachable: {detail}"),
            Self::Transport(detail) => write!(f, "transport failed: {detail}"),
            Self::Malformed(detail) => write!(f, "unreadable response: {detail}"),
            Self::Server { code, message } => {
                write!(f, "the request was refused: {code}: {message}")
            }
            Self::BothPathsFailed { cli, socket } => {
                write!(f, "{cli} (the socket was tried first: {socket})")
            }
        }
    }
}

#[cfg(test)]
mod tests;
