use herdr_picker_domain::Action;

use super::*;

#[test]
fn an_https_url_offers_timings_and_a_verified_certificate_check() {
    let rows = web_rows("https://example.com/path?q=1", "/opt/homebrew/bin/openssl");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].label, "HTTP timings and response headers");
    assert_eq!(
        rows[0].enter,
        Action::Insert("curl --globoff --proto =http,https --connect-timeout 10 --max-time 30 --silent --show-error --output /dev/null --dump-header - --write-out 'DNS: %{time_namelookup}s\nConnect: %{time_connect}s\nTLS: %{time_appconnect}s\nFirst byte: %{time_starttransfer}s\nTotal: %{time_total}s\nStatus: %{http_code}\nRemote: %{remote_ip}\n' --url 'https://example.com/path?q=1'".to_string())
    );
    assert_eq!(
        rows[1].enter,
        Action::Insert("/opt/homebrew/bin/openssl s_client -connect example.com:443 -servername example.com -showcerts -verify_return_error -verify_hostname example.com".to_string())
    );
}

#[test]
fn an_http_url_is_timed_only_and_an_address_is_verified_as_an_ip() {
    assert_eq!(web_rows("http://192.168.1.1:8080", "openssl").len(), 1);

    let rows = web_rows("https://[fe80::1]:8443/", "openssl");
    assert_eq!(
        rows[1].enter,
        Action::Insert("openssl s_client -connect '[fe80::1]:8443' -servername fe80::1 -showcerts -verify_return_error -verify_ip fe80::1".to_string())
    );
}

#[test]
fn nothing_typed_or_a_url_with_credentials_shows_one_row_saying_what_to_type() {
    let label = |typed: &str| web_rows(typed, "openssl")[0].label.clone();

    assert_eq!(label(""), "\u{1b}[2mType an HTTP or HTTPS URL\u{1b}[0m");
    assert_eq!(
        label("https://me:pw@example.com"),
        "\u{1b}[2mChoose an HTTP or HTTPS URL without credentials\u{1b}[0m"
    );
    assert_eq!(
        label("ftp://example.com"),
        "\u{1b}[2mChoose an HTTP or HTTPS URL without credentials\u{1b}[0m"
    );
    assert_eq!(label("https://bad;host"), "\u{1b}[2mInvalid host\u{1b}[0m");
}
