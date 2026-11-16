use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use pwe_adapters::{
    LaunchAgent, SystemdUser, daemon_log_file, daemon_socket_file, launch_agent_file,
    systemd_unit_file,
};
use pwe_application::{DaemonLaunch, SystemService};
use pwe_daemon::{
    Daemon, Started, Stopped, daemon_report, disable_daemon, enable_daemon, start_daemon,
    stop_daemon,
};

use super::{CommandError, failed};

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["daemon", "run"] => run(),
        ["daemon", "start"] => start(),
        ["daemon", "stop"] => stop(),
        ["daemon", "enable"] => enable(),
        ["daemon", "disable"] => disable(),
        ["daemon", "status"] => status(false),
        ["daemon", "status", "--output", "json"] => status(true),
        _ => return None,
    })
}

fn variable(name: &str) -> Option<String> {
    env::var(name).ok()
}

fn socket() -> PathBuf {
    daemon_socket_file(
        variable("XDG_RUNTIME_DIR").as_deref(),
        variable("XDG_STATE_HOME").as_deref(),
        variable("HOME").as_deref(),
    )
}

fn seconds_since_1970() -> Result<u64, CommandError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(failed)
}

fn run() -> Result<(), CommandError> {
    Daemon::claim(&socket(), seconds_since_1970()?)
        .map_err(failed)?
        .serve()
        .map_err(failed)
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
    match start_daemon(&program, &socket(), &log()).map_err(failed)? {
        Started::Now(pid) => println!("The pwe daemon started as process {pid}."),
        Started::AlreadyRunning(pid) => {
            println!("The pwe daemon is already running as process {pid}.")
        }
    }
    Ok(())
}

fn stop() -> Result<(), CommandError> {
    match stop_daemon(&socket()).map_err(failed)? {
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
    let pid = enable_daemon(system_service().as_ref(), &launch, &socket()).map_err(failed)?;
    println!(
        "The pwe daemon is enabled and running as process {pid}. It starts at every login and comes back after a crash."
    );
    Ok(())
}

fn disable() -> Result<(), CommandError> {
    disable_daemon(system_service().as_ref(), &socket()).map_err(failed)?;
    println!("The pwe daemon is disabled and stopped. It no longer starts at login.");
    Ok(())
}

fn status(as_json: bool) -> Result<(), CommandError> {
    let enabled = system_service().is_enabled();
    let report = daemon_report(&socket(), enabled).map_err(failed)?;
    if as_json {
        println!("{}", report.json());
    } else {
        print!("{}", report.plain_words());
    }
    Ok(())
}
