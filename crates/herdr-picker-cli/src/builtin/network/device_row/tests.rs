use super::*;
use crate::builtin::network::devices::Devices;
use crate::builtin::network::sightings::Sighting;
use crate::builtin::network::sightings::Source;

const CLOCK: Clock = Clock { now: 0, offset: 0 };

fn networks() -> Vec<Network> {
    vec![
        Network {
            interface: "en0".into(),
            address: Ipv4Addr::new(192, 168, 1, 61),
            prefix: 24,
        },
        Network {
            interface: "en7".into(),
            address: Ipv4Addr::new(10, 0, 0, 2),
            prefix: 16,
        },
    ]
}

fn one(source: Source, seen: Sighting) -> Device {
    let mut devices = Devices::default();
    devices.add(source, seen);
    devices.0.remove(0)
}

#[test]
fn a_device_row_pastes_its_address_and_offers_ssh_scans_its_web_page_and_wake_up() {
    let nas = one(
        Source::Bonjour,
        Sighting {
            ip: "192.168.1.20".into(),
            mac: "AA:BB:CC:00:11:22".into(),
            name: "nas".into(),
            host: "nas.local".into(),
            offers: vec!["_https._tcp".into()],
            ..Sighting::default()
        },
    );

    let row = device_row(&nas, "192.168.1.1", &networks(), CLOCK);

    assert_eq!(row.label, "nas  \u{1b}[2m192.168.1.20\u{1b}[0m");
    assert_eq!(row.group.as_deref(), Some("Bonjour"));
    assert_eq!(row.enter, Action::Insert("192.168.1.20".to_string()));
    let claimed: Vec<&str> = row.keys.iter().map(|(key, _)| key.shown()).collect();
    assert_eq!(
        claimed,
        ["ctrl-s", "ctrl-x", "alt-x", "ctrl-g", "ctrl-o", "ctrl-y"]
    );
    assert_eq!(
        row.keys[0].1,
        Action::Insert("ssh 192.168.1.20".to_string())
    );
    assert_eq!(row.keys[4].1, Action::Open("https://nas.local".to_string()));
    assert!(
        matches!(&row.keys[5].1, Action::Run(wake) if wake.ends_with(" network wake aa:bb:cc:00:11:22 192.168.1.255"))
    );
    assert!(
        matches!(&row.preview, Some(Preview::Command(argv)) if argv[1..4] == ["network", "ping", "192.168.1.20"])
    );
}

#[test]
fn a_device_off_this_macs_networks_is_woken_on_each_of_them_and_gear_shows_its_role() {
    let mut gear = one(
        Source::UniFi,
        Sighting {
            ip: "172.16.0.9".into(),
            mac: "aa:aa:aa:aa:aa:aa".into(),
            name: "Switch".into(),
            ..Sighting::default()
        },
    );
    gear.unifi.insert("category".into(), "devices".into());

    let row = device_row(&gear, "", &networks(), CLOCK);

    assert_eq!(
        row.label,
        "Switch  \u{1b}[2m172.16.0.9\u{1b}[0m  \u{1b}[38;2;203;166;247mUniFi gear\u{1b}[0m"
    );
    assert!(
        matches!(&row.keys[5].1, Action::Run(wake) if wake.ends_with(" 192.168.1.255 10.0.255.255"))
    );
}

#[test]
fn this_mac_previews_without_a_ping() {
    let mut me = one(
        Source::ThisMac,
        Sighting {
            ip: "192.168.1.61".into(),
            ..Sighting::default()
        },
    );
    me.this_mac = Some("Raith".into());

    let row = device_row(&me, "192.168.1.1", &[], CLOCK);

    assert_eq!(
        row.label,
        "Raith  \u{1b}[2m192.168.1.61\u{1b}[0m  \u{1b}[38;2;203;166;247mthis Mac\u{1b}[0m"
    );
    assert!(
        matches!(&row.preview, Some(Preview::Text(text)) if text.starts_with("Raith\nThis is the Mac you are using\n"))
    );
    assert_eq!(row.keys.len(), 5);
}
