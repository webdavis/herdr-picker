mod agents;
mod bash;
mod focused;
mod handover;
mod manifest;
mod pane_state;
mod panes;
mod popup;
mod rows;
mod shell;
mod view;
mod wire;

pub use agents::{
    WireTab, agent_list_request, agents_of, call_request, parse_status_change, tab_list_request,
    tabs_of,
};
pub use bash::bash_init;
pub use focused::{parse_context_pane, parse_focused_id, parse_focused_pane};
pub use handover::{
    Delivery, Handover, Instruction, PaneKind, Refusal, Underneath, Verb, delivery, guard,
    handover, parse_instruction,
};
pub use manifest::{PLUGIN_ID, render_manifest};
pub use pane_state::{PaneState, encode_pane_state, parse_pane_state};
pub use panes::{
    PaneInfo, enter_request, notify_request, open_request, pane_of, pane_request, screen_of,
    screen_request, swap_request, type_request,
};
pub use popup::{
    ACTIVE_WORKSPACE_VARIABLE, PICKER_BASE_VARIABLE, PICKER_CONFIG_VARIABLE, PICKER_NAME_VARIABLE,
    PICKER_RESULT_VARIABLE, PICKER_TARGET_VARIABLE, PopupPane, SOCKET_PATH_VARIABLE,
    layout_request, popup_request, window_of,
};
pub use rows::{
    PICKER_QUERY_VARIABLE, PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE, RowError, encode_action,
    encode_header, encode_herdr_call, encode_row, parse_action, parse_header, parse_herdr_call,
    parse_row,
};
pub use shell::{Shell, UnclosedQuote, instruction, shell_quote, shell_word, split_words};
pub use view::{encode_view_state, parse_view_state};
pub use wire::{
    ProtocolError, WireWorkspace, create_request, focus_request, list_request, result_of,
    workspaces_of,
};
