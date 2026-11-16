use std::path::Path;

use pwe_application::DesktopRoute;
use pwe_domain::{Corner, InboxLimits, NoticeSettings};
use serde::Deserialize;

use super::ConfigError;

const NOTICES_TABLE: &str = "[notices]";
const CORNERS: [(&str, Corner); 4] = [
    ("top-right", Corner::TopRight),
    ("top-left", Corner::TopLeft),
    ("bottom-right", Corner::BottomRight),
    ("bottom-left", Corner::BottomLeft),
];
const MACOS_ROUTES: [(&str, DesktopRoute); 4] = [
    ("auto", DesktopRoute::Auto),
    ("osascript", DesktopRoute::Osascript),
    ("terminal-notifier", DesktopRoute::TerminalNotifier),
    ("off", DesktopRoute::Off),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeConfig {
    pub corner: Corner,
    pub board: NoticeSettings,
    pub macos: DesktopRoute,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct NoticesTable {
    #[serde(default = "top_right")]
    corner: String,
    finished_seconds: Option<u64>,
    needs_you_seconds: Option<u64>,
    most_on_screen: Option<usize>,
    seen_after_seconds: Option<u64>,
    ask_session_every_seconds: Option<u64>,
    #[serde(default = "auto")]
    macos: String,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct InboxTable {
    keep_days: Option<u64>,
    most_rows: Option<usize>,
}

fn top_right() -> String {
    "top-right".to_string()
}

fn auto() -> String {
    "auto".to_string()
}

impl Default for NoticesTable {
    fn default() -> Self {
        Self {
            corner: top_right(),
            finished_seconds: None,
            needs_you_seconds: None,
            most_on_screen: None,
            seen_after_seconds: None,
            ask_session_every_seconds: None,
            macos: auto(),
        }
    }
}

impl NoticesTable {
    pub(super) fn settings(self, path: &Path) -> Result<NoticeConfig, ConfigError> {
        let defaults = NoticeSettings::default();
        Ok(NoticeConfig {
            corner: one_of(path, "corner", &self.corner, &CORNERS)?,
            board: NoticeSettings {
                finished_seconds: self.finished_seconds.unwrap_or(defaults.finished_seconds),
                needs_you_seconds: self.needs_you_seconds.unwrap_or(defaults.needs_you_seconds),
                most_on_screen: self.most_on_screen.unwrap_or(defaults.most_on_screen),
                seen_after_seconds: self
                    .seen_after_seconds
                    .unwrap_or(defaults.seen_after_seconds),
                ask_session_every_seconds: self
                    .ask_session_every_seconds
                    .unwrap_or(defaults.ask_session_every_seconds),
            },
            macos: one_of(path, "macos", &self.macos, &MACOS_ROUTES)?,
        })
    }
}

impl InboxTable {
    pub(super) fn limits(self) -> InboxLimits {
        let defaults = InboxLimits::default();
        InboxLimits {
            keep_days: self.keep_days.unwrap_or(defaults.keep_days),
            most_rows: self.most_rows.unwrap_or(defaults.most_rows),
        }
    }
}

fn one_of<T: Copy>(
    path: &Path,
    key: &'static str,
    word: &str,
    accepted: &[(&'static str, T)],
) -> Result<T, ConfigError> {
    accepted
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, value)| *value)
        .ok_or_else(|| ConfigError::UnknownWord {
            path: path.to_owned(),
            table: NOTICES_TABLE,
            key,
            word: word.to_string(),
            accepted: accepted.iter().map(|(name, _)| *name).collect(),
        })
}
