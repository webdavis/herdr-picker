use serde_json::json;

use super::*;

#[test]
fn strings_under_secret_keys_and_phone_numbers_are_hidden_and_the_rest_kept() {
    let account = json!({
        "user": "ann",
        "password": "hunter2",
        "api_key": "abc123",
        "apiKey": "abc123",
        "Authorization": "Bearer xyz",
        "refresh-token": "r1",
        "credentials": {"client_secret": "s", "region": "eu"},
        "contact": {"mobile": "555 0100", "note": "+1 (555) 123-4567"},
        "usage": {"input_tokens": 1200, "max_tokens": 4096},
        "born": "1990-04-01",
        "id": "1697040000"
    });

    assert_eq!(
        redacted(&account, None),
        json!({
            "user": "ann",
            "password": HIDDEN,
            "api_key": HIDDEN,
            "apiKey": HIDDEN,
            "Authorization": HIDDEN,
            "refresh-token": HIDDEN,
            "credentials": {"client_secret": HIDDEN, "region": "eu"},
            "contact": {"mobile": HIDDEN, "note": HIDDEN},
            "usage": {"input_tokens": 1200, "max_tokens": 4096},
            "born": "1990-04-01",
            "id": "1697040000"
        })
    );
}

#[test]
fn a_secret_copied_by_its_own_key_is_hidden_too() {
    assert_eq!(redacted(&json!("hunter2"), Some("password")), json!(HIDDEN));
    assert_eq!(redacted(&json!("ann"), Some("user")), json!("ann"));
    assert_eq!(
        redacted(&json!(["a", "b"]), Some("tokens")),
        json!([HIDDEN, HIDDEN])
    );
}
