mod cli;
mod clipboard;
mod config;
mod fzf;
mod herdr_socket;
mod history;
mod keyboard;
mod manifest;
mod pane_result;
mod picker_files;
mod picker_run;
mod popup_host;
mod process;
mod response;
mod screen;
mod socket;
mod state;

pub use cli::CliWorkspaceDirectory;
pub use clipboard::copy_to_clipboard;
pub use config::{ConfigError, PluginConfig, config_file, read_config};
pub use fzf::{
    Chrome, Helper, Placement, chooser_lines, error_footer, finder_actions, fzf_arguments,
    list_lines,
};
pub use herdr_socket::DEADLINE;
pub use history::FileWorkspaceHistory;
pub use keyboard::TerminalInput;
pub use manifest::{ManifestError, write_manifest};
pub use pane_result::{PaneWaitError, await_pane_result, write_pane_state};
pub use picker_files::{
    cached_rows, picker_error_file, read_run_error, read_view, remove_run_files, write_view,
};
pub use picker_run::{PickerRunError, run_picker};
pub use popup_host::SocketPopupHost;
pub use process::{spawn_detached, wait_for_process_exit};
pub use screen::{Layout, TerminalScreen};
pub use socket::SocketWorkspaceDirectory;
pub use state::{mru_file, picker_result_file, picker_rows_file, spawned_jump_log_file};

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;
