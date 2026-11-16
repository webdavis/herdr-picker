mod agent_programs;
mod agent_store;
mod bookmarks;
mod capture;
mod cli;
mod clipboard;
mod command_preview;
mod config;
mod daemon_client;
mod daemon_files;
pub mod finder;
mod fzf;
mod herdr_socket;
mod history;
mod inbox;
mod keyboard;
mod live_step;
mod mac_notifier;
mod manifest;
mod notice_pane;
mod pane_handover;
mod pane_result;
mod path_preview;
mod picker_files;
mod picker_run;
mod picker_text;
mod places;
mod popup_host;
mod private;
mod process;
mod program;
mod program_run;
mod quickfix;
mod response;
mod rex_event_stream;
mod rex_notice_screen;
mod rex_panes;
mod rex_program;
mod rex_whereabouts;
mod saved_contacts;
mod screen;
mod socket;
mod state;
mod system_service;
pub mod test_engine;
mod walk;

pub use agent_programs::{
    ClaudeCodeSessions, CodexSessions, LsofOpenFiles, OmpSessions, SystemProcessEnvironment,
};
pub use agent_store::{AgentSnapshot, read_queue, read_snapshot, write_queue, write_snapshot};
pub use bookmarks::{add_bookmark, read_bookmarks};
pub use capture::{Capture, read_captures, run_capture};
pub use cli::CliWorkspaceDirectory;
pub use clipboard::copy_to_clipboard;
pub use command_preview::command_preview;
pub use config::{
    AgentSettings, BlueBubblesSettings, ConfigError, PluginConfig, RexSettings, UnifiSettings,
    config_file, read_config, read_picker_config,
};
pub use daemon_client::{DaemonAskError, ask_daemon};
pub use daemon_files::{
    daemon_log_file, daemon_socket_file, launch_agent_file, pwe_state_folder, systemd_unit_file,
};
pub use fzf::{
    FzfStepFinder, Helper, Placement, SAVE_QUICKFIX, SEND_ROWS, SWITCH_SOURCE, change_footer,
    chooser_lines, finder_actions, leave, list_lines, unbound_action,
};
pub use herdr_socket::DEADLINE;
pub use history::FileWorkspaceHistory;
pub use inbox::{inbox_listing_json, prune_inbox_file, read_inbox, save_inbox_row};
pub use keyboard::TerminalInput;
pub use live_step::{is_live_step, rerun_step};
pub use mac_notifier::MacNotifier;
pub use manifest::{ManifestError, write_manifest};
pub use notice_pane::show_notice_pane;
pub use pane_handover::{
    HandoverError, PaneHost, Sent, SocketPanes, drain_queue, hand_over, send_prompt,
};
pub use pane_result::{PaneWaitError, await_pane_result, write_pane_state};
pub use path_preview::path_preview;
pub use picker_files::{
    cached_rows, picker_error_file, read_run_error, read_step_header, read_view,
    remove_runs_whose_process_is_gone, write_view,
};
pub use picker_run::{PickerRunError, run_picker};
pub use picker_text::{error_footer, help_page, step_footer};
pub use places::{recent_places, record_place_without_a_lock};
pub use popup_host::SocketPopupHost;
pub use process::{spawn_detached, wait_for_process_exit};
pub use program::{bare_program, on_path};
pub use quickfix::{open_quickfix, read_quickfix, save_quickfix, send_quickfix};
pub use rex_event_stream::RexEventStream;
pub use rex_notice_screen::RexNoticeScreen;
pub use rex_panes::RexPanes;
pub use rex_program::rex_program;
pub use rex_whereabouts::RexWhereabouts;
pub use saved_contacts::{contacts_file, save_contacts, saved_contacts};
pub use screen::{Layout, TerminalScreen};
pub use socket::SocketWorkspaceDirectory;
pub use state::{
    bookmarks_file, captures_dir, inbox_file, mru_file, picker_result_file, picker_rows_file,
    places_file, plugin_state_dir, quickfix_file, send_queue_file, snapshot_file,
    spawned_jump_log_file,
};
pub use system_service::{LaunchAgent, SystemdUser};
pub use walk::{Entry, Walk, walk};

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
#[cfg(test)]
#[path = "tests/fake_program.rs"]
mod fake_program;
#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;
