use herdr_picker_domain::{Action, Preview};

use super::*;

#[test]
fn a_capture_is_read_by_summary_packets_conversations_protocols_or_a_typed_filter() {
    let rows = analysis("/caps/a b.pcap");

    assert_eq!(rows.len(), 5);
    assert_eq!(
        rows[2].enter,
        Action::Insert("tshark -n -r '/caps/a b.pcap' -q -z conv,tcp -z conv,udp".to_string())
    );
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "Capture file summary\ncapinfos '/caps/a b.pcap'\nEnter prepares the command."
                .to_string()
        ))
    );
    assert!(matches!(&rows[4].enter, Action::Next(call) if call.name == "capture-filter"));
    assert_eq!(
        filtered("/caps/a.pcap", "tcp.port == 443").enter,
        Action::Insert("tshark -n -r /caps/a.pcap -Y 'tcp.port == 443'".to_string())
    );
    assert_eq!(filtered("/caps/a.pcap", " ").enter, Action::Nothing);
}
