mod support;

use std::fs;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::{TempHome, json_of, succeeded};

const FAKE_CLAUDE: &str =
    "#!/bin/sh\necho called >> \"$HOME/claude-calls\"\ncat \"$HOME/claude-answer\"\n";
const PATIENCE: Duration = Duration::from_secs(30);

fn answer(name: &str) -> String {
    format!(
        r#"[{{"pid":108,"cwd":"/work/project","kind":"background","sessionId":"99999999-9999-4999-8999-999999999999","name":"{name}","status":"busy","state":"working"}}]"#
    )
}

fn home_with_a_fake_claude(name: &str) -> TempHome {
    let home = TempHome::new(name)
        .with_no_rex_app()
        .with_fake_program("claude", FAKE_CLAUDE);
    fs::write(
        home.folder.join(".config/pwe/config.toml"),
        "[rex]\napp-name = \"Absent Terminal\"\n\n[agents]\nprograms = [\"claude-code\"]\nrefresh-seconds = 3600\n",
    )
    .unwrap_or_else(|failure| panic!("{failure}"));
    home
}

fn claude_answers(home: &TempHome, name: &str) {
    fs::write(home.folder.join("claude-answer"), answer(name))
        .unwrap_or_else(|failure| panic!("{failure}"));
}

fn claude_calls(home: &TempHome) -> usize {
    fs::read_to_string(home.folder.join("claude-calls"))
        .unwrap_or_default()
        .lines()
        .count()
}

fn without_times(listing: &str) -> Value {
    let mut listing = json_of(listing);
    for session in listing["agent-sessions"]
        .as_array_mut()
        .into_iter()
        .flatten()
    {
        session["state-since"] = json!(0);
    }
    listing
}

fn agents(home: &TempHome) -> String {
    succeeded(home.pwe(&["agents"]))
}

#[test]
fn pwe_agents_reads_by_itself_when_no_daemon_runs() {
    let home = home_with_a_fake_claude("agents-alone");
    claude_answers(&home, "first-answer");

    let listing = without_times(&agents(&home));

    assert_eq!(listing["agent-sessions"][0]["name"], json!("first-answer"));
    assert_eq!(claude_calls(&home), 1);
}

#[test]
fn pwe_agents_asks_the_daemon_when_it_runs_and_prints_the_same_listing() {
    let home = home_with_a_fake_claude("agents-daemon");
    claude_answers(&home, "first-answer");
    let by_itself = agents(&home);
    succeeded(home.pwe(&["daemon", "start"]));
    let deadline = Instant::now() + PATIENCE;
    while claude_calls(&home) < 2 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    claude_answers(&home, "second-answer");

    let mut through_the_daemon = agents(&home);
    while !through_the_daemon.contains("first-answer") && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
        through_the_daemon = agents(&home);
    }
    succeeded(home.pwe(&["daemon", "stop"]));
    let after_the_daemon = agents(&home);

    assert_eq!(
        without_times(&through_the_daemon),
        without_times(&by_itself)
    );
    assert_eq!(
        through_the_daemon.trim().replace(char::is_numeric, ""),
        by_itself.trim().replace(char::is_numeric, "")
    );
    assert!(
        after_the_daemon.contains("second-answer"),
        "{after_the_daemon}"
    );
}
