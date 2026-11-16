use super::*;

fn settings(url: &str, server_name: Option<&str>) -> UnifiSettings {
    UnifiSettings {
        url: url.to_string(),
        keychain_service: "UniFi API key".to_string(),
        ca_file: Some("~/certs/unifi.pem".to_string()),
        server_name: server_name.map(str::to_string),
    }
}

#[test]
fn a_controller_at_an_address_is_asked_by_its_certificates_name() {
    let reached = controller(
        &settings("https://192.168.1.1/", Some("unifi.local")),
        Some("/Users/me"),
    )
    .expect("a controller");

    assert_eq!(
        reached,
        Controller {
            base: "https://unifi.local:443/proxy/network/integration/v1".to_string(),
            connect_to: Some("unifi.local:443:192.168.1.1:443".to_string()),
            ca_file: Some("/Users/me/certs/unifi.pem".to_string()),
        }
    );
    assert_eq!(
        curl_args(&reached, "/sites", 200),
        [
            "curl",
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "4",
            "--max-redirs",
            "0",
            "--proto",
            "=https",
            "--header",
            "@-",
            "--cacert",
            "/Users/me/certs/unifi.pem",
            "--connect-to",
            "unifi.local:443:192.168.1.1:443",
            "--url",
            "https://unifi.local:443/proxy/network/integration/v1/sites?offset=200&limit=200",
        ]
    );
}

#[test]
fn a_controller_url_keeps_its_port_and_path() {
    let reached = controller(&settings("https://gw.lan:8443/unifi", None), Some("/h"));

    assert_eq!(
        reached.map(|reached| (reached.base, reached.connect_to)),
        Ok((
            "https://gw.lan:8443/unifi/proxy/network/integration/v1".to_string(),
            None
        ))
    );
}

#[test]
fn a_plain_http_or_credentialed_url_is_refused() {
    for url in [
        "http://192.168.1.1",
        "https://admin:pw@192.168.1.1",
        "https://gw?x=1",
        "https://",
    ] {
        assert_eq!(
            controller(&settings(url, None), Some("/h")),
            Err("UniFi requires an HTTPS controller URL without credentials".to_string()),
            "{url}"
        );
    }
}
