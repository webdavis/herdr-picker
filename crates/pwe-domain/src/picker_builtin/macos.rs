use super::{BuiltinPicker, Live};

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
