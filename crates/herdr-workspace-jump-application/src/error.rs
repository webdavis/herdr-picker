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
            Self::Unreachable(detail) => write!(f, "herdr unreachable: {detail}"),
            Self::Transport(detail) => write!(f, "herdr transport failed: {detail}"),
            Self::Malformed(detail) => write!(f, "unreadable herdr response: {detail}"),
            Self::Server { code, message } => {
                write!(f, "herdr refused the request: {code}: {message}")
            }
            Self::BothPathsFailed { cli, socket } => {
                write!(f, "{cli} (the socket was tried first: {socket})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_renders_on_one_line() {
        for error in [
            JumpError::Unreachable("no such file".to_string()),
            JumpError::Transport("timed out".to_string()),
            JumpError::Malformed("not JSON".to_string()),
            JumpError::Server {
                code: "not_found".to_string(),
                message: "gone".to_string(),
            },
            JumpError::BothPathsFailed {
                cli: "herdr unreachable: no binary".to_string(),
                socket: "herdr unreachable: no socket".to_string(),
            },
        ] {
            let rendered = error.to_string();
            assert!(!rendered.contains('\n'), "multi-line: {rendered}");
            assert!(!rendered.is_empty());
        }
    }
}
