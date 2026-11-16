use super::*;

fn job() -> SystemJob<'static> {
    SystemJob {
        program: Path::new("/opt/pwe & co/bin/pwe"),
        path_variable: "/opt/bin:/usr/bin:/bin",
        log: Path::new("/h/.local/state/pwe/daemon.log"),
    }
}

fn without_spacing(text: &str) -> String {
    text.split_whitespace().collect()
}

#[test]
fn the_launchd_job_returns_after_a_crash_but_not_after_stop() {
    let plist = without_spacing(&launchd_job(&job()));

    assert!(
        plist.contains("<key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict>"),
        "{plist}"
    );
    assert!(plist.contains("<key>RunAtLoad</key><true/>"), "{plist}");
}

#[test]
fn the_launchd_job_runs_the_absolute_pwe_with_the_callers_path() {
    let plist = launchd_job(&job());

    assert!(
        plist.contains("<key>Label</key>\n  <string>pwe-daemon</string>"),
        "{plist}"
    );
    assert!(
        without_spacing(&plist).contains(
            "<key>ProgramArguments</key><array><string>/opt/pwe&amp;co/bin/pwe</string><string>daemon</string><string>run</string></array>"
        ),
        "{plist}"
    );
    assert!(
        plist.contains("<key>PATH</key>\n    <string>/opt/bin:/usr/bin:/bin</string>"),
        "{plist}"
    );
    assert!(
        plist.contains(
            "<key>StandardErrorPath</key>\n  <string>/h/.local/state/pwe/daemon.log</string>"
        ),
        "{plist}"
    );
}

#[test]
fn the_systemd_unit_restarts_on_failure_and_starts_with_the_user() {
    let unit = systemd_unit(&job());

    assert!(
        unit.contains("ExecStart=\"/opt/pwe & co/bin/pwe\" daemon run\n"),
        "{unit}"
    );
    assert!(unit.contains("Restart=on-failure\n"), "{unit}");
    assert!(
        unit.contains("Environment=\"PATH=/opt/bin:/usr/bin:/bin\"\n"),
        "{unit}"
    );
    assert!(unit.contains("WantedBy=default.target\n"), "{unit}");
}

#[test]
fn the_systemd_unit_escapes_quotes_and_percent_signs() {
    let unit = systemd_unit(&SystemJob {
        program: Path::new("/opt/100%\"pwe\"/pwe"),
        path_variable: "/a%b",
        log: Path::new("/l"),
    });

    assert!(
        unit.contains("ExecStart=\"/opt/100%%\\\"pwe\\\"/pwe\" daemon run\n"),
        "{unit}"
    );
    assert!(unit.contains("Environment=\"PATH=/a%%b\"\n"), "{unit}");
}
