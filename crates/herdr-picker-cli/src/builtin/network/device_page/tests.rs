use serde_json::json;

use super::*;
use crate::builtin::network::devices::Devices;
use crate::builtin::network::sightings::Sighting;
use crate::builtin::network::sightings::Source;

const CLOCK: Clock = Clock {
    now: 1_792_404_464,
    offset: 0,
};

fn listed(sightings: Vec<(Source, Sighting)>) -> Vec<Device> {
    let mut devices = Devices::default();
    sightings
        .into_iter()
        .for_each(|(source, seen)| devices.add(source, seen));
    devices.link_uplinks();
    devices.0
}

#[test]
fn a_wifi_client_reads_as_a_sentence_then_its_details_services_and_unifi_record() {
    let gear = json!({"id": "g1", "category": "devices", "model": "UDR7", "state": "ONLINE", "firmwareVersion": "4.1", "firmwareUpdatable": true});
    let client = json!({"id": "c1", "category": "clients", "type": "WIRELESS", "uplinkDeviceId": "g1", "connectedAt": "2026-10-19T07:07:44Z", "access": {"type": "GUEST", "authorized": false}, "sites": 1, "osName": "iOS"});
    let devices = listed(vec![
        (
            Source::UniFi,
            Sighting {
                ip: "192.168.1.1".into(),
                name: "Dresden".into(),
                unifi: gear.as_object().cloned().unwrap(),
                ..Sighting::default()
            },
        ),
        (
            Source::UniFi,
            Sighting {
                ip: "192.168.1.30".into(),
                mac: "AA:BB:CC:00:11:22".into(),
                name: "Phone".into(),
                unifi: client.as_object().cloned().unwrap(),
                ..Sighting::default()
            },
        ),
        (
            Source::Bonjour,
            Sighting {
                ip: "192.168.1.30".into(),
                name: "iPhone".into(),
                host: "iphone.local".into(),
                offers: vec!["_companion-link._tcp".into()],
                ..Sighting::default()
            },
        ),
    ]);

    assert_eq!(
        sentence(&devices[0], CLOCK),
        "UniFi router (UDR7), online, firmware 4.1, update available."
    );
    assert_eq!(
        page(&devices[1], "192.168.1.1", CLOCK),
        "Phone
Wi-Fi, connected through Dresden (192.168.1.1), since 2026-10-19 07:07 (3 hours)

Details
Also known as      iPhone (Bonjour)
IP address         192.168.1.30
MAC address        aa:bb:cc:00:11:22
Hostname           iphone.local
Connection         Wi-Fi
Connected through  Dresden (192.168.1.1)
Connected since    2026-10-19 07:07 (3 hours)
Gateway            192.168.1.1
Found by           UniFi, Bonjour

Services announced
Apple device  _companion-link._tcp

From UniFi
UniFi record ID  c1
UniFi access     guest network, not yet authorized
osName           iOS"
    );
}

#[test]
fn a_device_only_one_source_saw_says_so() {
    let devices = listed(vec![(
        Source::Scan,
        Sighting {
            ip: "192.168.1.50".into(),
            ..Sighting::default()
        },
    )]);

    assert_eq!(sentence(&devices[0], CLOCK), "Seen by Scan.");
}
