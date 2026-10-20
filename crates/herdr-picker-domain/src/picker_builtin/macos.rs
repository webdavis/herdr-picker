use super::{BuiltinPicker, Live};

// The Mac's own apps, which the shell suite had as its macOS pickers.
pub(super) const APPLE: BuiltinPicker = BuiltinPicker {
    name: "apple",
    key: "ctrl-t m",
    description: "Messages through BlueBubbles, Contacts, and Shortcuts",
    sources: &["messages", "contacts", "shortcuts"],
    live: Live::Never,
};

pub(super) const TABS: BuiltinPicker = BuiltinPicker {
    name: "tabs",
    key: "ctrl-t w",
    description: "Open Arc tabs, by Space",
    sources: &[],
    live: Live::Never,
};
