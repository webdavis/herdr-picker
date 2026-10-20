use std::path::Path;

use herdr_picker_domain::{Action, PickerCall};

use super::*;
use crate::builtin::tests::Tree;

#[test]
fn each_interface_opens_the_capture_settings_for_it() {
    let rows = capture_interfaces("lo0 en0\n");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].label, "Capture on en0");
    assert_eq!(
        rows[1].enter,
        Action::Next(PickerCall {
            name: "capture-command".into(),
            args: vec!["en0".into()],
            state: None,
            query: None,
        })
    );
    assert_eq!(capture_interfaces("")[0].label, "Capture on en0");
}

#[test]
fn with_nothing_typed_the_capture_is_twenty_packets_with_tshark() {
    let (prompt, row) = capture_command("en0", "", Path::new("/work"), None);

    assert_eq!(
        prompt,
        "Type settings such as host=nas.local proto=tcp port=443 count=50 seconds=10 tool=tcpdump file=out.pcap, then press Enter."
    );
    assert_eq!(row.label, "Packet capture");
    assert_eq!(
        row.enter,
        Action::Insert("sudo tshark -n -i en0 -c 20".to_string())
    );
}

#[test]
fn typed_settings_bound_and_filter_the_capture() {
    let (_, row) = capture_command(
        "en1",
        "host=nas.local proto=tcp port=443 count=50 seconds=10 file=out.pcap",
        Path::new("/work"),
        None,
    );
    assert_eq!(
        row.enter,
        Action::Insert(
            "sudo tshark -n -i en1 -c 50 -a duration:10 -w /work/out.pcap -f 'host nas.local and tcp and port 443'"
                .to_string()
        )
    );

    let (_, dump) = capture_command(
        "en0",
        "tool=tcpdump proto=udp file=~/a.pcap",
        Path::new("/work"),
        Some(Path::new("/home/me")),
    );
    assert_eq!(
        dump.enter,
        Action::Insert("sudo tcpdump -n -i en0 -c 20 -w /home/me/a.pcap udp".to_string())
    );
}

#[test]
fn settings_that_cannot_run_are_named_instead() {
    let tree = Tree::new("capture-builder", &["taken.pcap"]);
    let label = |typed: &str| capture_command("en0", typed, &tree.0, None).1.label;

    assert_eq!(
        label("file=taken.pcap"),
        "\u{1b}[2mCapture output already exists; choose a new filename\u{1b}[0m"
    );
    assert_eq!(
        label("tool=tcpdump seconds=5"),
        "\u{1b}[2mUse tshark for a duration-limited capture\u{1b}[0m"
    );
    assert_eq!(
        label("proto=arp"),
        "\u{1b}[2mChoose any, tcp, udp, icmp or icmp6\u{1b}[0m"
    );
    assert_eq!(
        label("tool=wireshark"),
        "\u{1b}[2mChoose tshark or tcpdump\u{1b}[0m"
    );
    assert_eq!(label("speed=1"), "\u{1b}[2mUnknown setting speed\u{1b}[0m");
    assert_eq!(
        label("nas.local"),
        "\u{1b}[2mType settings as name=value\u{1b}[0m"
    );
    assert_eq!(label("host=a;b"), "\u{1b}[2mInvalid host\u{1b}[0m");
}
