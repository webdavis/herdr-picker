use super::*;

const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<nmaprun scanner="nmap" args="nmap -sn -oX /s/network-scan.xml -oN /s/network-scan.txt 192.168.1.0/24" start="1792404464" version="7.95">
<host><status state="up" reason="arp-response"/>
<address addr="192.168.1.1" addrtype="ipv4"/>
<address addr="74:AC:B9:01:02:03" addrtype="mac" vendor="Ubiquiti &amp; Co"/>
<hostnames>
<hostname name="router.lan" type="PTR"/>
</hostnames>
</host>
<host><status state="up" reason="localhost-response"/>
<address addr="192.168.1.61" addrtype="ipv4"/>
<hostnames>
</hostnames>
</host>
<runstats><hosts up="2" down="254" total="256"/></runstats>
</nmaprun>
"#;

const CLOCK: Clock = Clock {
    now: 1_792_404_464 + 600,
    offset: 0,
};

#[test]
fn a_scan_lists_each_host_with_its_mac_vendor_and_name() {
    let scan = parse_scan(XML, "Nmap done\n", CLOCK.now).expect("a scan");

    assert_eq!(scan.network, "192.168.1.0/24");
    assert_eq!(scan.at, 1_792_404_464);
    assert_eq!(
        scan.hosts,
        vec![
            Sighting {
                ip: "192.168.1.1".into(),
                mac: "74:AC:B9:01:02:03".into(),
                vendor: "Ubiquiti & Co".into(),
                name: "router.lan".into(),
                ..Sighting::default()
            },
            Sighting {
                ip: "192.168.1.61".into(),
                ..Sighting::default()
            },
        ]
    );
}

#[test]
fn a_scan_older_than_a_day_is_forgotten() {
    assert_eq!(parse_scan(XML, "", 1_792_404_464 + 24 * 3600), None);
    assert_eq!(parse_scan("not xml", "", 0), None);
}

#[test]
fn the_last_scan_row_prepares_its_command_and_copies_its_report() {
    let scan = parse_scan(XML, "Nmap done\n", CLOCK.now).expect("a scan");
    let row = scan_row(&scan, CLOCK);

    assert_eq!(
        row.label,
        "\u{1b}[1mLast scan\u{1b}[0m  \u{1b}[38;2;137;180;250m192.168.1.0/24\u{1b}[0m  \u{1b}[2m2 hosts up at 10:07\u{1b}[0m"
    );
    assert_eq!(
        row.enter,
        Action::Insert("sudo nmap -sn 192.168.1.0/24".to_string())
    );
    assert_eq!(
        row.keys[1],
        (key("alt-y"), Action::Copy("Nmap done".to_string()))
    );
}
