use herdr_picker_domain::{Action, PickerCall};

use super::*;

#[test]
fn each_dns_record_type_opens_the_lookup_for_it() {
    let rows = dns_records();

    let labels: Vec<&str> = rows.iter().map(|row| row.label.as_str()).collect();
    assert_eq!(
        labels,
        ["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SOA", "SRV", "CAA"]
    );
    assert_eq!(
        rows[3].enter,
        Action::Next(PickerCall {
            name: "dns-lookup".into(),
            args: vec!["MX".into()],
            state: None,
            query: None,
        })
    );
}

#[test]
fn a_dns_lookup_asks_dig_with_an_optional_server() {
    let (prompt, row) = dns_lookup("MX", "example.com").expect("a record");
    assert_eq!(
        prompt,
        "Type the host and an optional DNS server, then press Enter."
    );
    assert_eq!(row.label, "DNS MX");
    assert_eq!(
        row.enter,
        Action::Insert("dig +time=2 +tries=1 example.com MX".to_string())
    );

    let (_, served) = dns_lookup("A", "example.com 1.1.1.1").expect("a record");
    assert_eq!(
        served.enter,
        Action::Insert("dig +time=2 +tries=1 @1.1.1.1 example.com A".to_string())
    );

    let (_, blank) = dns_lookup("A", "").expect("a record");
    assert_eq!(blank.label, "\u{1b}[2mType the host\u{1b}[0m");
    let (_, wrong) = dns_lookup("A", "bad;host").expect("a record");
    assert_eq!(wrong.label, "\u{1b}[2mInvalid host\u{1b}[0m");
    assert!(dns_lookup("PTR", "example.com").is_none());
}

#[test]
fn iperf_modes_open_the_step_that_builds_their_command() {
    let rows = iperf_modes();

    let labels: Vec<&str> = rows.iter().map(|row| row.label.as_str()).collect();
    assert_eq!(
        labels,
        ["client", "reverse", "bidirectional", "udp", "server"]
    );
    assert!(
        matches!(&rows[4].enter, Action::Next(call) if call.name == "iperf-command" && call.args == ["server"])
    );
}

#[test]
fn an_iperf_client_takes_the_server_then_optional_seconds_port_and_bitrate() {
    let (prompt, row) = iperf_command("client", "nas.local").expect("a mode");
    assert_eq!(
        prompt,
        "Type the iperf3 server, then optionally seconds and port, then press Enter."
    );
    assert_eq!(row.label, "iperf3 client");
    assert_eq!(
        row.enter,
        Action::Insert("iperf3 --json --port 5201 --client nas.local --time 10".to_string())
    );

    let (_, reverse) = iperf_command("reverse", "nas.local 30 5202").expect("a mode");
    assert_eq!(
        reverse.enter,
        Action::Insert(
            "iperf3 --json --port 5202 --client nas.local --time 30 --reverse".to_string()
        )
    );

    let (prompt, udp) = iperf_command("udp", "nas.local 5 5201 50M").expect("a mode");
    assert_eq!(
        prompt,
        "Type the iperf3 server, then optionally seconds, port and bitrate, then press Enter."
    );
    assert_eq!(
        udp.enter,
        Action::Insert(
            "iperf3 --json --port 5201 --client nas.local --time 5 --udp --bitrate 50M".to_string()
        )
    );

    let (_, both) = iperf_command("bidirectional", "nas.local").expect("a mode");
    assert_eq!(
        both.enter,
        Action::Insert(
            "iperf3 --json --port 5201 --client nas.local --time 10 --bidir".to_string()
        )
    );
}

#[test]
fn an_iperf_server_takes_only_a_port_and_bad_values_are_named() {
    let (prompt, server) = iperf_command("server", "").expect("a mode");
    assert_eq!(prompt, "Type an optional port, then press Enter.");
    assert_eq!(
        server.enter,
        Action::Insert("iperf3 --json --port 5201 --server --one-off".to_string())
    );

    let (_, missing) = iperf_command("client", "").expect("a mode");
    assert_eq!(missing.label, "\u{1b}[2mType the iperf3 server\u{1b}[0m");
    let (_, rate) = iperf_command("udp", "nas.local 5 5201 fast").expect("a mode");
    assert_eq!(rate.label, "\u{1b}[2mInvalid bitrate\u{1b}[0m");
    let (_, seconds) = iperf_command("client", "nas.local 0").expect("a mode");
    assert_eq!(
        seconds.label,
        "\u{1b}[2mduration must be a positive integer\u{1b}[0m"
    );
    assert!(iperf_command("tcp", "").is_none());
}
