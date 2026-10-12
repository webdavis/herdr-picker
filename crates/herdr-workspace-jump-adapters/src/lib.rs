mod cli;
mod config;
mod herdr_socket;
mod history;
mod keyboard;
mod manifest;
mod process;
mod response;
mod screen;
mod socket;
mod state;

pub use cli::CliWorkspaceDirectory;
pub use config::{ConfigError, config_file, read_jump_targets};
pub use herdr_socket::DEADLINE;
pub use history::FileWorkspaceHistory;
pub use keyboard::TerminalKeys;
pub use manifest::{ManifestError, write_manifest};
pub use process::{spawn_detached, wait_for_process_exit};
pub use screen::TerminalScreen;
pub use socket::SocketWorkspaceDirectory;
pub use state::{log_file, state_file};

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;
