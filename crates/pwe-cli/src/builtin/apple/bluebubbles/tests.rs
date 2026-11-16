use serde_json::json;

use super::*;
use crate::builtin::program::Ran;

fn ran(stdout: &str, stderr: &str, succeeded: bool) -> Ran {
    Ran {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        succeeded,
    }
}

#[test]
fn a_server_is_reached_over_https_or_over_http_on_this_mac_only() {
    let remote = server("https://messages.example.net:1234/bb/").expect("a server");
    assert_eq!(remote.url, "https://messages.example.net:1234/bb");
    assert!(server("http://localhost:1234").is_ok());
    assert_eq!(
        server("http://messages.example.net").err().as_deref(),
        Some("BlueBubbles requires HTTPS outside this Mac.")
    );
    for refused in [
        "https://me:pw@host",
        "https://host/?password=x",
        "https://",
        "host:1234",
    ] {
        assert_eq!(
            server(refused).err().as_deref(),
            Some(
                "BlueBubbles URL must contain only its scheme, host, port, and optional base path."
            ),
            "{refused}"
        );
    }
}

#[test]
fn the_password_reaches_curl_on_stdin_and_never_in_its_arguments() {
    let remote = server("https://messages.example.net").expect("a server");
    let args = curl_args(&remote, &json!({"limit": 200}));

    assert_eq!(
        args,
        [
            "curl",
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "20",
            "--max-redirs",
            "0",
            "--proto",
            "=https",
            "--header",
            "Content-Type: application/json",
            "--data-binary",
            r#"{"limit":200}"#,
            "--config",
            "-",
        ]
    );
    assert_eq!(
        curl_config(&remote, "/chat/query", "p&ss wörd\""),
        "url = \"https://messages.example.net/api/v1/chat/query?password=p%26ss%20w%C3%B6rd%22\"\n"
    );
}

#[test]
fn a_failed_request_says_why_without_what_curl_was_sent() {
    assert_eq!(
        answer(&ran(
            "",
            "curl: (22) The requested URL returned error: 401",
            false
        )),
        Err(
            "BlueBubbles request failed (HTTP 401). Check its URL and server password.".to_string()
        )
    );
    assert_eq!(
        answer(&ran("", "curl: (7) Failed to connect", false)),
        Err("BlueBubbles is unavailable or returned an invalid response. Check its server and picker configuration.".to_string())
    );
    assert_eq!(
        answer(&ran(r#"{"status": 200}"#, "", true)),
        Err("BlueBubbles returned an unexpected response.".to_string())
    );
    assert_eq!(
        answer(&ran(r#"{"data": [], "metadata": {"total": 0}}"#, "", true)),
        Ok(json!({"data": [], "metadata": {"total": 0}}))
    );
}
