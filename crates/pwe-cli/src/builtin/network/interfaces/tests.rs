use super::*;

const IFCONFIG: &str = "lo0: flags=8049<UP,LOOPBACK,RUNNING,MULTICAST> mtu 16384
\tinet 127.0.0.1 netmask 0xff000000
\tinet6 ::1 prefixlen 128
en0: flags=8863<UP,BROADCAST,SMART,RUNNING,SIMPLEX,MULTICAST> mtu 1500
\tether 02:00:00:00:00:00
\tinet6 fe80::10f1:f827:d2ba:e5b8%en0 prefixlen 64 secured scopeid 0xc
\tinet 192.168.1.61 netmask 0xffffff00 broadcast 192.168.1.255
utun4: flags=8051<UP,POINTOPOINT,RUNNING,MULTICAST> mtu 1280
\tinet 100.101.1.2 --> 100.101.1.2 netmask 0xffffffff
";

#[test]
fn ifconfig_gives_this_macs_addresses_and_the_networks_worth_scanning() {
    assert_eq!(
        own_addresses(IFCONFIG),
        [
            "127.0.0.1",
            "::1",
            "fe80::10f1:f827:d2ba:e5b8",
            "192.168.1.61",
            "100.101.1.2"
        ]
    );
    let found = networks(IFCONFIG);
    assert_eq!(
        found,
        vec![Network {
            interface: "en0".to_string(),
            address: Ipv4Addr::new(192, 168, 1, 61),
            prefix: 24
        }]
    );
    assert_eq!(found[0].range(), "192.168.1.0/24");
    assert_eq!(found[0].broadcast(), Ipv4Addr::new(192, 168, 1, 255));
    assert!(found[0].holds(Ipv4Addr::new(192, 168, 1, 7)));
    assert!(!found[0].holds(Ipv4Addr::new(192, 168, 2, 7)));
}

#[test]
fn the_gateway_is_the_default_routes() {
    let routes = "Routing tables\n\nInternet:\nDestination        Gateway            Flags               Netif Expire\ndefault            192.168.1.1        UGScg                 en0\n";
    assert_eq!(gateway(routes), "192.168.1.1");
    assert_eq!(gateway("Routing tables\n"), "");
}
