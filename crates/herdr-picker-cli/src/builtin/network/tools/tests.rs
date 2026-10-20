use super::*;

#[test]
fn a_ping_reads_as_its_round_trip_or_no_reply() {
    let answered = "PING 192.168.1.1 (192.168.1.1): 56 data bytes\n64 bytes from 192.168.1.1: icmp_seq=0 ttl=64 time=3.612 ms\n";
    assert_eq!(ping_time(answered), "4 ms");
    assert_eq!(ping_time("Request timeout for icmp_seq 0\n"), "no reply");
}

#[test]
fn the_resolvers_name_for_an_address() {
    assert_eq!(
        resolved("name: nas.local\nip_address: 192.168.1.20\n\n"),
        Some("nas.local".to_string())
    );
    assert_eq!(resolved(""), None);
}

#[test]
fn a_magic_packet_is_six_ff_bytes_then_the_mac_sixteen_times() {
    let packet = magic_packet("aa:bb:cc:00:11:22").expect("a packet");

    assert_eq!(packet.len(), 102);
    assert_eq!(
        packet[..12],
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xaa, 0xbb, 0xcc, 0x00, 0x11, 0x22
        ]
    );
    assert_eq!(packet[96..], [0xaa, 0xbb, 0xcc, 0x00, 0x11, 0x22]);
    assert_eq!(magic_packet("not a mac"), None);
}
