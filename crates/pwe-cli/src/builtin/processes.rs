use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use super::clock::local_offset;
use super::page::notice;
use super::program::run;
use super::rows::RowWriter;
use super::{Places, written};
use crate::command::{CommandError, failed};

mod docker;
mod logs;
mod ps;
mod services;

const SERVICE_DEADLINE: Duration = Duration::from_secs(8);

pub(super) fn processes(
    source: &str,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let listed = match source {
        "services" => services(),
        "logs" => logs::log_kinds(),
        _ => {
            let (listed, sockets) = thread::scope(|scope| {
                let sockets =
                    scope.spawn(|| run(&["lsof", "-nP", "-i", "-FpcnT"], SERVICE_DEADLINE));
                let listed = run(&["ps", "-axo", "pid=,ppid=,user=,comm="], SERVICE_DEADLINE);
                (
                    listed,
                    sockets.join().ok().and_then(Result::ok).unwrap_or_default(),
                )
            });
            ps::process_rows(&listed.map_err(failed)?, &sockets)
        }
    };
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

// Homebrew, launchd and Docker, asked at once; one that cannot answer says why in a row.
fn services() -> Vec<super::PickerRow> {
    let asked: [(&str, &[&str]); 4] = [
        ("brew", &["brew", "services", "list", "--json"]),
        ("launchd", &["launchctl", "list"]),
        ("docker", &["docker", "ps", "-a", "--format", "{{json .}}"]),
        (
            "compose",
            &["docker", "compose", "ps", "--all", "--format", "json"],
        ),
    ];
    let answers: Vec<_> = thread::scope(|scope| {
        let asking: Vec<_> = asked
            .iter()
            .map(|(_, argv)| scope.spawn(|| run(argv, SERVICE_DEADLINE)))
            .collect();
        asking
            .into_iter()
            .map(|answer| answer.join().unwrap_or(Err("stopped".into())))
            .collect()
    });
    let mut seen = Vec::new();
    let mut rows = Vec::new();
    for ((source, _), answer) in asked.iter().zip(answers) {
        let listed = answer.and_then(|said| match *source {
            "brew" => services::brew_rows(&said),
            "launchd" => Ok(services::launchd_rows(&said)),
            _ => docker::docker_rows(&said, &mut seen),
        });
        match listed {
            Ok(listed) => rows.extend(listed),
            Err(why) => rows.push(notice(&format!("{source} unavailable"), &why)),
        }
    }
    rows
}

// The last hour can be long, so its rows go out as `log` prints them.
pub(super) fn unified_logs(rows: &mut RowWriter<impl Write>) -> Result<(), CommandError> {
    let mut log = Command::new("/usr/bin/log")
        .args(["show", "--style", "ndjson", "--last", "1h", "--no-pager"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(failed)?;
    let lines = BufReader::new(
        log.stdout
            .take()
            .ok_or_else(|| failed("log printed nothing"))?,
    );
    let shown = lines
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| logs::log_row(&line))
        .try_for_each(|row| rows.row(&row));
    let _ = log.kill();
    let _ = log.wait();
    written(shown)
}

pub(super) fn crashes(
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let mut folders: Vec<PathBuf> = places
        .home
        .iter()
        .map(|home| home.join("Library/Logs/DiagnosticReports"))
        .collect();
    folders.push(PathBuf::from("/Library/Logs/DiagnosticReports"));
    let listed = logs::crash_rows(&folders, local_offset());
    written(listed.iter().try_for_each(|row| rows.row(row)))
}
