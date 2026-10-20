use super::{BuiltinPicker, Live};

// Every picker on its key, and every key binding: herdr-picker's own and the owner's chord table.
pub(super) const PICKERS: BuiltinPicker = BuiltinPicker {
    name: "pickers",
    key: "ctrl-t ;",
    description: "Every picker by its key, or every key binding in the shell",
    sources: &["pickers", "bindings"],
    live: Live::Never,
};
