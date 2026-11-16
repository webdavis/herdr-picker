use super::*;

#[test]
fn counts_and_ports_are_positive_whole_numbers_and_ports_stop_at_65535() {
    assert_eq!(validate("port", " 443 "), Ok("443".to_string()));
    assert_eq!(validate("count", "20"), Ok("20".to_string()));
    assert_eq!(
        validate("port", "70000"),
        Err("port must be a positive integer up to 65535".to_string())
    );
    assert_eq!(
        validate("duration", "0"),
        Err("duration must be a positive integer".to_string())
    );
}

#[test]
fn a_range_must_name_its_prefix_and_is_kept_as_its_network() {
    assert_eq!(
        validate("cidr", "192.168.1.77/24"),
        Ok("192.168.1.0/24".to_string())
    );
    assert_eq!(validate("cidr", "fd00::1/64"), Ok("fd00::/64".to_string()));
    assert_eq!(
        validate("cidr", "192.168.1.0"),
        Err("Choose an explicit subnet, such as 192.168.1.0/24".to_string())
    );
    assert_eq!(
        validate("cidr", "10.0.0.0/33"),
        Err("Invalid cidr".to_string())
    );
}

#[test]
fn hosts_are_names_or_addresses_and_nothing_a_shell_would_read() {
    assert_eq!(validate("host", "nas.local"), Ok("nas.local".to_string()));
    assert_eq!(
        validate("host", "fe80::1%en0"),
        Ok("fe80::1%en0".to_string())
    );
    assert_eq!(
        validate("host", "a;rm -rf"),
        Err("Invalid host".to_string())
    );
    assert_eq!(
        validate("host", "fe80::zz"),
        Err("Invalid host".to_string())
    );
    assert_eq!(validate("ip", "10.0.0.300"), Err("Invalid ip".to_string()));
    assert_eq!(validate("ip", "10.0.0.3"), Ok("10.0.0.3".to_string()));
}

#[test]
fn any_other_value_must_be_present_and_printable() {
    assert_eq!(
        validate("display filter", "tcp.port == 443"),
        Ok("tcp.port == 443".to_string())
    );
    assert_eq!(
        validate("display filter", ""),
        Err("Invalid display filter".to_string())
    );
    assert_eq!(
        validate("display filter", "a\u{7}b"),
        Err("Invalid display filter".to_string())
    );
}
