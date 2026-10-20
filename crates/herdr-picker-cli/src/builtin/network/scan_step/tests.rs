use std::net::Ipv4Addr;

use super::*;

#[test]
fn the_scan_step_runs_nmap_on_a_network_or_a_typed_range_and_alt_x_only_prepares() {
    let files = ScanFiles::in_folder(Path::new("/state"));
    let networks = [Network {
        interface: "en0".into(),
        address: Ipv4Addr::new(192, 168, 1, 61),
        prefix: 24,
    }];

    let rows = scan_rows(&networks, "", false, &files);
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].enter,
        Action::Run("mkdir -p /state && sudo -k nmap -sn 192.168.1.0/24 -oX /state/network-scan.xml -oN /state/network-scan.txt".to_string())
    );
    assert_eq!(rows[1].label, "\u{1b}[2mType a range by hand\u{1b}[0m");

    let typed = scan_rows(&networks, "fd00::5/64", true, &files);
    assert_eq!(
        typed[0].enter,
        Action::Insert("sudo nmap -6 -sn fd00::/64".to_string())
    );
    assert_eq!(typed.len(), 2);

    let wrong = scan_rows(&[], "10.0.0.1", false, &files);
    assert_eq!(
        wrong[0].label,
        "\u{1b}[2mChoose an explicit subnet, such as 192.168.1.0/24\u{1b}[0m"
    );
}
