use std::path::PathBuf;

use herdr_picker_domain::{Action, Preview};

use super::*;

fn nas() -> SshHost {
    SshHost {
        alias: "nas".into(),
        file: PathBuf::from("/home/me/.ssh/config"),
        settings: vec!["Hostname: 192.168.1.5".into()],
    }
}

#[test]
fn ssh_aliases_prepare_ssh_and_hosts_file_names_paste_their_address() {
    let rows = host_rows(
        &[nas()],
        Some("127.0.0.1 localhost\n# comment\n192.168.1.9 printer printer.lan # office\n::1\n"),
        None,
    );

    let labels: Vec<&str> = rows.iter().map(|row| row.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "nas  \u{1b}[2mSSH\u{1b}[0m",
            "localhost  \u{1b}[2m127.0.0.1\u{1b}[0m",
            "printer  \u{1b}[2m192.168.1.9\u{1b}[0m",
            "printer.lan  \u{1b}[2m192.168.1.9\u{1b}[0m",
            "\u{1b}[2mTailscale status is unavailable\u{1b}[0m",
        ]
    );
    assert_eq!(rows[0].enter, Action::Insert("ssh nas".to_string()));
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "nas\nssh nas\nStatic host alias from /home/me/.ssh/config; SSH evaluates the full configuration when run.\nHostname: 192.168.1.5\nEnter prepares the command.".to_string()
        ))
    );
    assert_eq!(rows[0].group.as_deref(), Some("SSH"));
    assert_eq!(rows[2].enter, Action::Insert("192.168.1.9".to_string()));
    assert_eq!(rows[2].group.as_deref(), Some("/etc/hosts"));
    assert_eq!(
        rows[2].preview,
        Some(Preview::Text(
            "Name  printer\nIP    192.168.1.9".to_string()
        ))
    );
}

#[test]
fn tailscale_peers_paste_their_first_address_and_show_their_status() {
    let status = r#"{
        "Self": {"ID": "1", "HostName": "mac", "DNSName": "mac.tail.ts.net.", "TailscaleIPs": ["100.64.0.1", "fd7a::1"], "OS": "macOS", "Online": true},
        "Peer": {"key": {"ID": "2", "HostName": "pi", "TailscaleIPs": [], "Online": false, "LastSeen": "2026-10-01T00:00:00Z"}}
    }"#;

    let rows = host_rows(&[], None, Some(status));

    assert_eq!(rows[0].label, "\u{1b}[2m/etc/hosts is unavailable\u{1b}[0m");
    assert_eq!(
        rows[1].label,
        "mac.tail.ts.net.  \u{1b}[2m100.64.0.1\u{1b}[0m"
    );
    assert_eq!(rows[1].enter, Action::Insert("100.64.0.1".to_string()));
    assert_eq!(rows[1].group.as_deref(), Some("Tailscale"));
    assert_eq!(
        rows[1].preview,
        Some(Preview::Text(
            "HostName      mac\nDNSName       mac.tail.ts.net.\nTailscaleIPs  100.64.0.1, fd7a::1\nOS            macOS\nOnline        true".to_string()
        ))
    );
    assert_eq!(rows[2].label, "pi");
    assert_eq!(rows[2].enter, Action::Insert("pi".to_string()));
    assert_eq!(rows.len(), 3);
    assert_eq!(
        host_rows(&[], Some(""), Some("not json"))[0].label,
        "\u{1b}[2mTailscale status is unavailable\u{1b}[0m"
    );
}

#[test]
fn a_name_with_shell_characters_is_pasted_quoted() {
    let status = r#"{"Peer": {"key": {"HostName": "Me's iPad", "TailscaleIPs": []}}}"#;
    let odd = SshHost {
        alias: "nas$(x)".into(),
        ..nas()
    };

    let rows = host_rows(&[odd], Some("192.168.1.9 a$b\n"), Some(status));

    let pasted: Vec<&Action> = rows.iter().map(|row| &row.enter).collect();
    assert_eq!(
        pasted,
        [
            &Action::Insert("ssh 'nas$(x)'".to_string()),
            &Action::Insert("192.168.1.9".to_string()),
            &Action::Insert(r"'Me'\''s iPad'".to_string()),
        ]
    );
}
