mod bash;
mod focused;
mod manifest;
mod pane_state;
mod popup;
mod rows;
mod shell;
mod view;
mod wire;

pub use bash::bash_init;
pub use focused::parse_focused_id;
pub use manifest::{PLUGIN_ID, render_manifest};
pub use pane_state::{PaneState, encode_pane_state, parse_pane_state};
pub use popup::{
    ACTIVE_WORKSPACE_VARIABLE, PICKER_CONFIG_VARIABLE, PICKER_NAME_VARIABLE,
    PICKER_RESULT_VARIABLE, PopupPane, SOCKET_PATH_VARIABLE, layout_request, popup_request,
    window_of,
};
pub use rows::{
    PICKER_STATE_VARIABLE, RowError, encode_action, encode_row, parse_action, parse_row,
};
pub use shell::{Shell, UnclosedQuote, instruction, shell_quote, shell_word, split_words};
pub use view::{encode_view_state, parse_view_state};
pub use wire::{
    ProtocolError, WireWorkspace, create_request, focus_request, list_request, result_of,
    workspaces_of,
};
