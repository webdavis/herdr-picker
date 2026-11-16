use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopRoute {
    Auto,
    Osascript,
    TerminalNotifier,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopNotifierError {
    NoProgram,
    ProgramMissing(String),
    Refused { program: String, detail: String },
}

impl fmt::Display for DesktopNotifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProgram => write!(
                formatter,
                "neither terminal-notifier nor osascript was found on PATH, so no macOS notification was sent"
            ),
            Self::ProgramMissing(program) => write!(
                formatter,
                "{program} was not found on PATH, so no macOS notification was sent"
            ),
            Self::Refused { program, detail } => write!(formatter, "{program} refused: {detail}"),
        }
    }
}

pub trait DesktopNotifier {
    fn notify(&self, title: &str, text: &str) -> Result<(), DesktopNotifierError>;
}
