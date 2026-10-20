use serde_json::json;

use super::*;
use crate::builtin::network::naming::role;

fn seen(ip: &str, mac: &str, name: &str) -> Sighting {
    Sighting {
        ip: ip.to_string(),
        mac: mac.to_string(),
        name: name.to_string(),
        ..Sighting::default()
    }
}

fn unifi(fields: Value) -> Map<String, Value> {
    fields.as_object().cloned().expect("an object")
}

#[test]
fn sightings_of_one_mac_or_one_address_merge_into_one_device() {
    let mut devices = Devices::default();
    devices.add(
        Source::ThisMac,
        seen("192.168.1.40", "A4:83:E7:AA:BB:CC", ""),
    );
    devices.add(
        Source::UniFi,
        seen("192.168.1.40", "a4:83:e7:aa:bb:cc", "Office Mac"),
    );
    devices.add(
        Source::Bonjour,
        Sighting {
            host: "office.local".to_string(),
            offers: vec!["_ssh._tcp".to_string()],
            ..seen("192.168.1.40", "", "Office")
        },
    );
    devices.add(Source::Scan, seen("192.168.1.50", "", ""));

    assert_eq!(devices.0.len(), 2);
    let office = &devices.0[0];
    assert_eq!(office.mac, "a4:83:e7:aa:bb:cc");
    assert_eq!(
        office.sources,
        [Source::ThisMac, Source::UniFi, Source::Bonjour]
    );
    assert_eq!(display_name(office), "Office Mac");
    assert_eq!(office.host, "office.local");
    assert_eq!(office.offers, ["_ssh._tcp"]);
    assert_eq!(display_name(&devices.0[1]), "192.168.1.50");
}

#[test]
fn every_address_of_this_mac_becomes_one_device_named_for_the_computer() {
    let mut devices = Devices::default();
    devices.add(Source::Scan, seen("192.168.1.9", "", ""));
    devices.add(Source::ThisMac, seen("fe80::1", "02:00:00:00:00:01", ""));
    devices.add(Source::Bonjour, seen("192.168.1.61", "", "raith"));

    devices.mark_this_mac(
        &["192.168.1.61".to_string(), "fe80::1".to_string()],
        "Raith",
    );

    assert_eq!(devices.0.len(), 2);
    let me = &devices.0[1];
    assert_eq!(me.ips, ["fe80::1", "192.168.1.61"]);
    assert_eq!(display_name(me), "Raith");
    assert_eq!(role(me), "this Mac");
    assert_eq!(primary_ip(me), "192.168.1.61");
}

#[test]
fn a_client_learns_the_unifi_gear_it_connects_through_and_gear_names_its_role() {
    let mut devices = Devices::default();
    devices.add(Source::UniFi, Sighting { unifi: unifi(json!({"id": "g1", "category": "devices", "model": "UDR7", "features": ["switching", "accessPoint"]})), ..seen("192.168.1.1", "", "Dresden") });
    devices.add(
        Source::UniFi,
        Sighting {
            unifi: unifi(json!({"id": "c1", "category": "clients", "uplinkDeviceId": "g1"})),
            ..seen("192.168.1.30", "", "Phone")
        },
    );

    devices.link_uplinks();

    assert_eq!(role(&devices.0[0]), "router, access point, switch");
    assert_eq!(
        devices.0[1].through,
        Some(("Dresden".to_string(), "192.168.1.1".to_string()))
    );
    assert_eq!(role(&devices.0[1]), "");
}
