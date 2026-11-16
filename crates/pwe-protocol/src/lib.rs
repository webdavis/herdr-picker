mod agent_programs;
mod agents;
mod bash;
mod daemon_line;
mod daemon_status;
mod doctor_report;
mod focused;
mod handover;
mod manifest;
mod pane_state;
mod panes;
mod popup;
mod rex;
mod rex_event;
mod rex_whereabouts;
mod rows;
mod shell;
mod system_job;
pub mod test_engine;
mod view;
mod wire;

pub use agent_programs::{
    agent_sessions_json, agent_sessions_value, claude_code_sessions, codex_session,
    omp_held_session_ids, omp_session, open_files_in_lsof_listing, variable_in_environ_file,
    variable_in_ps_listing,
};
pub use agents::{
    WireTab, agent_list_request, agents_of, call_request, parse_status_change, tab_list_request,
    tabs_of,
};
pub use bash::bash_init;
pub use daemon_line::{
    DaemonError, DaemonErrorKind, DaemonRequest, daemon_reply_line, daemon_request_line,
    parse_daemon_reply, parse_daemon_request,
};
pub use daemon_status::{
    DaemonStatus, Listens, RexLink, daemon_status_result, parse_daemon_status,
};
pub use doctor_report::{doctor_report_json, doctor_report_words};
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
pub use rex::{
    KeyFacts, OpenedLayer, block_label_request, close_layer_request, float_request,
    focused_block_in_view, notice_float_request, opened_block, opened_layer, parse_key_facts,
    rex_lua, rex_pane, rex_pane_status, rex_reported_state, rex_session_ids, rex_status_record,
    rex_terminal_panes, session_request, window_in_view,
};
pub use rex_event::{
    ForegroundProcess, ProgramStatus, RexBlock, RexEvent, RexEventError, parse_rex_event,
};
pub use rex_whereabouts::parse_rex_whereabouts;
pub use rows::{
    PICKER_QUERY_VARIABLE, PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE, RowError, encode_action,
    encode_header, encode_herdr_call, encode_row, parse_action, parse_header, parse_herdr_call,
    parse_row,
};
pub use shell::{Shell, UnclosedQuote, instruction, shell_quote, shell_word, split_words};
pub use system_job::{SystemJob, launchd_job, systemd_unit};
pub use view::{encode_view_state, parse_view_state};
pub use wire::{
    ProtocolError, WireWorkspace, create_request, focus_request, list_request, result_of,
    workspaces_of,
};
