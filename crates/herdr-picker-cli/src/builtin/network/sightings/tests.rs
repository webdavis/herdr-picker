use super::*;

#[test]
fn arp_neighbours_carry_their_name_and_mac_and_incomplete_ones_are_dropped() {
    let table = "router.lan (192.168.1.1) at 74:ac:b9:1:2:3 on en0 ifscope [ethernet]
? (192.168.1.40) at a4:83:e7:aa:bb:cc on en0 ifscope [ethernet]
? (192.168.1.99) at (incomplete) on en0 ifscope [ethernet]
";
    let seen = arp(table);

    assert_eq!(seen.len(), 2);
    assert_eq!(
        seen[0],
        Sighting {
            ip: "192.168.1.1".to_string(),
            mac: "74:ac:b9:1:2:3".to_string(),
            name: "router.lan".to_string(),
            neighbour: "answered recently".to_string(),
            ..Sighting::default()
        }
    );
    assert_eq!(seen[1].name, "");
}

#[test]
fn ndp_neighbours_keep_their_state_and_this_macs_own_addresses_are_dropped() {
    let table =
        "Neighbor                                Linklayer Address  Netif Expire    St Flgs Prbs
fe80::10f1:f827:d2ba:e5b8%en0           2:0:0:0:0:0          en0 permanent R
fe80::1%lo0                             (incomplete)         lo0 permanent R
fe80::4a5:1b2c:aa:1%en0                 4a:5:1b:2c:0:1       en0 23h59m58s S  R
";
    let seen = ndp(table);

    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].ip, "fe80::4a5:1b2c:aa:1");
    assert_eq!(seen[0].mac, "4a:5:1b:2c:0:1");
    assert_eq!(seen[0].neighbour, "23h59m58s S R");
}

#[test]
fn bonjour_instances_and_where_they_are_reached() {
    let browse = "Browsing for _ssh._tcp.local.
DATE: ---Mon 19 Oct 2026---
 6:14:22.495  ...STARTING...
Timestamp     A/R    Flags  if Domain               Service Type         Instance Name
 6:14:22.646  Add        3   1 local.               _ssh._tcp.           raith
 6:14:22.646  Add        2  12 local.               _ssh._tcp.           Living Room TV
";
    assert_eq!(
        browsed(browse),
        vec![
            ("raith".to_string(), "local.".to_string()),
            ("Living Room TV".to_string(), "local.".to_string())
        ]
    );
    let lookup =
        " 6:14:23.829  raith._ssh._tcp.local. can be reached at raith.local.:22 (interface 12)\n";
    assert_eq!(reached(lookup), Some("raith.local".to_string()));
    assert_eq!(reached(" 6:14:23.829  ...STARTING...\n"), None);
}
