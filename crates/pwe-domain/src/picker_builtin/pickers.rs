use super::{BuiltinPicker, Live};

pub(super) const PICKERS: BuiltinPicker = BuiltinPicker {
    name: "pickers",
    key: "ctrl-t ;",
    description: "Every picker by its key, or every key binding in the shell",
    sources: &["pickers", "bindings"],
    live: Live::Never,
};
