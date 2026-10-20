use herdr_picker_domain::{Action, Preview};

use super::*;

#[test]
fn the_catalog_lists_every_command_and_the_selectors_this_picker_has() {
    let rows = command_rows();
    let titled = |title: &str| {
        rows.iter()
            .find(|row| row.label.starts_with(&format!("{title}  ")))
            .expect(title)
    };

    assert_eq!(rows.len(), 46);
    assert_eq!(
        titled("Hostname").label,
        "Hostname  \u{1b}[2mhostname\u{1b}[0m"
    );
    assert_eq!(
        titled("Hostname").enter,
        Action::Insert("hostname".to_string())
    );
    assert_eq!(
        titled("Hostname").preview,
        Some(Preview::Text(
            "Hostname\nhostname\nPrint the name of this computer.\nEnter prepares the command."
                .to_string()
        ))
    );
    assert_eq!(
        titled("URL timing and certificate").enter,
        Action::Source("web checks".to_string())
    );
    assert!(
        matches!(&titled("Probe host and port").enter, Action::Next(call) if call.args == ["port"])
    );
    let opens = |title: &str| match &titled(title).enter {
        Action::Next(call) if call.args.is_empty() => call.name.clone(),
        other => panic!("{title} opens {other:?}"),
    };
    assert_eq!(opens("Saved hosts"), "network-hosts");
    assert_eq!(opens("DNS record inspection"), "network-dns");
    assert_eq!(opens("Build a packet capture"), "network-capture");
    assert_eq!(opens("Throughput test"), "network-iperf");
    assert_eq!(
        titled("Saved hosts").preview,
        Some(Preview::Text(
            "Search static SSH hosts, hosts-file names and Tailscale peers.\nEnter opens the selector."
                .to_string()
        ))
    );
}

#[test]
fn a_commands_blanks_are_typed_in_order_and_checked() {
    let (prompt, row) = command_step("port", "nas.local 22").expect("a step");
    assert_eq!(
        prompt,
        "Type the host and port, separated by spaces, then press Enter."
    );
    assert_eq!(
        row.enter,
        Action::Insert("nc -vz -w3 nas.local 22".to_string())
    );

    let (_, missing) = command_step("port", "nas.local").expect("a step");
    assert_eq!(missing.label, "\u{1b}[2mType the port\u{1b}[0m");

    let (_, wrong) = command_step("port", "nas.local 99999").expect("a step");
    assert_eq!(
        wrong.label,
        "\u{1b}[2mport must be a positive integer up to 65535\u{1b}[0m"
    );
}

#[test]
fn an_ipv6_range_scans_with_nmap_6() {
    let (_, row) = command_step("discovery", "fd00::1/64").expect("a step");

    assert_eq!(
        row.enter,
        Action::Insert("nmap -6 -sn -n fd00::/64".to_string())
    );
    assert!(command_step("nope", "").is_none());
}
