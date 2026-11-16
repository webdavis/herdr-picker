use std::env;
use std::path::PathBuf;

use pwe_adapters::{
    LaunchAgent, SystemdUser, daemon_log_file, launch_agent_file, systemd_unit_file,
};
use pwe_application::{DaemonLaunch, SystemService};
use pwe_daemon::{
    Daemon, Started, Stopped, daemon_report, disable_daemon, enable_daemon, seconds_since_1970,
    start_daemon, stop_daemon,
};

use super::{CommandError, daemon_socket, failed, prints_json};
use crate::agent_sessions::Surroundings;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["daemon", "run"] => run(),
        ["daemon", "start"] => start(),
        ["daemon", "stop"] => stop(),
        ["daemon", "enable"] => enable(),
        ["daemon", "disable"] => disable(),
        ["daemon", "status", options @ ..] => prints_json(options).and_then(status),
        _ => return None,
    })
}

fn variable(name: &str) -> Option<String> {
    env::var(name).ok()
}

fn run() -> Result<(), CommandError> {
    let mut daemon = Daemon::claim(&daemon_socket(), seconds_since_1970()).map_err(failed)?;
    let surroundings = Surroundings::of_this_process();
    daemon.follow_rex(surroundings.rex_program());
    let refresh_seconds = surroundings.agents.refresh_seconds.max(1);
    daemon.follow_agents(
        move || surroundings.read_agent_sources(),
        refresh_seconds,
        Vec::new(),
    );
    daemon.serve().map_err(failed)
}

fn log() -> PathBuf {
    daemon_log_file(
        variable("XDG_STATE_HOME").as_deref(),
        variable("HOME").as_deref(),
    )
}

fn system_service() -> Box<dyn SystemService> {
    let home = variable("HOME");
    if cfg!(target_os = "macos") {
        Box::new(LaunchAgent::at(launch_agent_file(home.as_deref())))
    } else {
        Box::new(SystemdUser::at(systemd_unit_file(
            variable("XDG_CONFIG_HOME").as_deref(),
            home.as_deref(),
        )))
    }
}

fn start() -> Result<(), CommandError> {
    let program = env::current_exe().map_err(failed)?;
    match start_daemon(&program, &daemon_socket(), &log()).map_err(failed)? {
        Started::Now(pid) => println!("The pwe daemon started as process {pid}."),
        Started::AlreadyRunning(pid) => {
            println!("The pwe daemon is already running as process {pid}.")
        }
    }
    Ok(())
}

fn stop() -> Result<(), CommandError> {
    match stop_daemon(&daemon_socket()).map_err(failed)? {
        Stopped::Now => println!("The pwe daemon stopped."),
        Stopped::WasNotRunning => println!("The pwe daemon was not running."),
    }
    Ok(())
}

fn enable() -> Result<(), CommandError> {
    let program = env::current_exe().map_err(failed)?;
    let path_variable = variable("PATH").unwrap_or_default();
    let log = log();
    let launch = DaemonLaunch {
        program: &program,
        path_variable: &path_variable,
        log: &log,
    };
    let pid =
        enable_daemon(system_service().as_ref(), &launch, &daemon_socket()).map_err(failed)?;
    println!(
        "The pwe daemon is enabled and running as process {pid}. It starts at every login and comes back after a crash."
    );
    Ok(())
}

fn disable() -> Result<(), CommandError> {
    disable_daemon(system_service().as_ref(), &daemon_socket()).map_err(failed)?;
    println!("The pwe daemon is disabled and stopped. It no longer starts at login.");
    Ok(())
}

fn status(as_json: bool) -> Result<(), CommandError> {
    let enabled = system_service().is_enabled();
    let report = daemon_report(&daemon_socket(), enabled).map_err(failed)?;
    if as_json {
        println!("{}", report.json());
    } else {
        print!("{}", report.plain_words());
    }
    Ok(())
}
