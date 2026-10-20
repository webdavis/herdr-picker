mod bookmarks;
mod capture;
mod cli;
mod clipboard;
mod command_preview;
mod config;
mod fzf;
mod herdr_socket;
mod history;
mod keyboard;
mod live_step;
mod manifest;
mod pane_result;
mod path_preview;
mod picker_files;
mod picker_run;
mod popup_host;
mod private;
mod process;
mod program;
mod quickfix;
mod response;
mod screen;
mod socket;
mod state;
mod walk;

pub use bookmarks::{add_bookmark, read_bookmarks};
pub use capture::{Capture, read_captures, run_capture};
pub use cli::CliWorkspaceDirectory;
pub use clipboard::copy_to_clipboard;
pub use command_preview::command_preview;
pub use config::{
    ConfigError, PluginConfig, UnifiSettings, config_file, read_config, read_picker_config,
};
pub use fzf::{
    Chrome, Helper, Placement, SAVE_QUICKFIX, SWITCH_SOURCE, change_footer, chooser_lines,
    error_footer, finder_actions, fzf_arguments, help_page, leave, list_lines, step_footer,
    unbound_action,
};
pub use herdr_socket::DEADLINE;
pub use history::FileWorkspaceHistory;
pub use keyboard::TerminalInput;
pub use live_step::{is_live_step, rerun_step, save_step_command};
pub use manifest::{ManifestError, write_manifest};
pub use pane_result::{PaneWaitError, await_pane_result, write_pane_state};
pub use path_preview::path_preview;
pub use picker_files::{
    cached_rows, picker_error_file, read_run_error, read_step_header, read_view, remove_run_files,
    write_view,
};
pub use picker_run::{Picked, PickerRunError, run_picker};
pub use popup_host::SocketPopupHost;
pub use process::{spawn_detached, wait_for_process_exit};
pub use program::{bare_program, on_path};
pub use quickfix::{open_quickfix, read_quickfix, save_quickfix, send_quickfix};
pub use screen::{Layout, TerminalScreen};
pub use socket::SocketWorkspaceDirectory;
pub use state::{
    bookmarks_file, captures_dir, mru_file, picker_result_file, picker_rows_file, quickfix_file,
    spawned_jump_log_file,
};
pub use walk::{Entry, Walk, walk};

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;
