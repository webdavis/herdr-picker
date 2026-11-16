use std::path::Path;

const SYSTEM_JOB_LABEL: &str = "pwe-daemon";

pub struct SystemJob<'a> {
    pub program: &'a Path,
    pub path_variable: &'a str,
    pub log: &'a Path,
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn systemd_quoted(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%");
    format!("\"{escaped}\"")
}

pub fn launchd_job(job: &SystemJob) -> String {
    let program = xml_text(&job.program.to_string_lossy());
    let path_variable = xml_text(job.path_variable);
    let log = xml_text(&job.log.to_string_lossy());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{SYSTEM_JOB_LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{program}</string>
    <string>daemon</string>
    <string>run</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>{path_variable}</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <dict>
    <key>SuccessfulExit</key>
    <false/>
  </dict>
  <key>StandardErrorPath</key>
  <string>{log}</string>
</dict>
</plist>
"#
    )
}

pub fn systemd_unit(job: &SystemJob) -> String {
    let program = systemd_quoted(&job.program.to_string_lossy());
    let path_setting = systemd_quoted(&format!("PATH={}", job.path_variable));
    format!(
        "[Unit]
Description=pwe daemon

[Service]
ExecStart={program} daemon run
Environment={path_setting}
Restart=on-failure

[Install]
WantedBy=default.target
"
    )
}

#[cfg(test)]
mod tests;
