use super::*;

const UNIFI: &str = r#"
[pickers.network.unifi]
url = "https://192.168.1.1"
keychain-service = "UniFi API key"
ca-file = "~/certs/unifi.pem"
server-name = "unifi.local"
"#;

#[test]
fn the_network_pickers_unifi_table_names_the_controller_and_the_keychain_item() {
    let sandbox = TempDirectory::new("unifi");
    let config = read_config(&sandbox.write(UNIFI)).expect("a readable config");

    assert_eq!(
        config.unifi,
        Some(UnifiSettings {
            url: "https://192.168.1.1".to_string(),
            keychain_service: "UniFi API key".to_string(),
            ca_file: Some("~/certs/unifi.pem".to_string()),
            server_name: Some("unifi.local".to_string()),
        })
    );
    assert!(config.pickers.iter().any(|picker| picker.name == "network"));
}

#[test]
fn without_a_unifi_table_unifi_is_not_configured() {
    let sandbox = TempDirectory::new("no-unifi");
    let config = read_config(&sandbox.write("[pickers.network]\nkey = \"ctrl-t n\"\n"))
        .expect("a readable config");

    assert_eq!(config.unifi, None);
}

#[test]
fn a_unifi_table_under_another_picker_is_refused() {
    let (refusal, rendered) = refusal_for(
        "misplaced-unifi",
        &UNIFI.replace("pickers.network.unifi", "pickers.files.unifi"),
    );

    assert!(
        matches!(refusal, ConfigError::Misplaced { .. }),
        "{rendered}"
    );
    assert!(
        rendered.ends_with("[pickers.files.unifi] belongs under [pickers.network]"),
        "{rendered}"
    );
}
