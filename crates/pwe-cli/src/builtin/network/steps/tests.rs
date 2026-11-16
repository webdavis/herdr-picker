use crate::builtin::Asked;
use crate::builtin::tests::{Tree, asked};

fn typed(name: &str, arguments: &[&str], query: &str, tree: &Tree) -> String {
    let asking = Asked {
        query,
        ..Asked::default()
    };
    asked(name, arguments, &asking, &tree.places("home")).expect("rows")
}

#[test]
fn the_catalog_steps_print_their_prompt_and_the_command_typed() {
    let tree = Tree::new("network-steps", &["traces/a.pcap"]);

    let dns = typed("dns-lookup", &["MX"], "example.com", &tree);
    assert!(dns.contains("Type the host and an optional DNS server"));
    assert!(dns.contains("dig +time=2 +tries=1 example.com MX"));

    let iperf = typed("iperf-command", &["server"], "", &tree);
    assert!(iperf.contains("iperf3 --json --port 5201 --server --one-off"));

    let capture = typed("capture-command", &["en0"], "count=5", &tree);
    assert!(capture.contains("sudo tshark -n -i en0 -c 5"));

    assert!(typed("network-dns", &[], "", &tree).contains("CNAME"));
    assert!(typed("network-iperf", &[], "", &tree).contains("bidirectional"));
}

#[test]
fn alt_f_opens_a_folder_whose_captures_then_list() {
    let tree = Tree::new("network-folder-steps", &["traces/a.pcap"]);

    let folder = typed("capture-folder", &[], "traces", &tree);
    assert!(folder.contains("Type the folder to search for captures"));
    assert!(folder.contains(&format!("Search {}", tree.path("traces"))));

    let listed = typed("captures-in", &[&tree.path("traces")], "", &tree);
    assert!(listed.contains("a.pcap"));
    let unknown = asked(
        "dns-lookup",
        &["PTR"],
        &Asked::default(),
        &tree.places("home"),
    );
    assert!(unknown.is_err());
}
