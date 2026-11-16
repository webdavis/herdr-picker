use serde_json::json;

use super::*;

#[test]
fn json_inside_a_string_is_read_as_the_object_or_array_it_holds() {
    assert_eq!(
        decoded(r#" {"user": "ann", "roles": ["admin"]} "#),
        Some((
            vec![Decoding::Json],
            json!({"user": "ann", "roles": ["admin"]})
        ))
    );
    assert_eq!(
        decoded("[1, 2]"),
        Some((vec![Decoding::Json], json!([1, 2])))
    );
}

#[test]
fn base64_is_read_as_its_text_and_further_as_the_json_that_text_holds() {
    assert_eq!(
        decoded("aGVsbG8gdGhlcmUsIHdvcmxk"),
        Some((vec![Decoding::Base64], json!("hello there, world")))
    );
    assert_eq!(
        decoded("eyJ1c2VyIjoiYW5uIn0="),
        Some((
            vec![Decoding::Base64, Decoding::Json],
            json!({"user": "ann"})
        ))
    );
}

#[test]
fn plain_words_ids_and_binary_payloads_are_left_as_they_are() {
    for text in [
        "hello",
        "abcdefghijklmnop",
        "0123456789abcdef0123456789abcdef",
        "/w==/w==/w==/w==",
        "{not json",
        "42",
        "\"quoted\"",
        "",
    ] {
        assert_eq!(decoded(text), None, "{text}");
    }
}

#[test]
fn base64_decodes_the_standard_alphabet_with_or_without_padding() {
    assert_eq!(base64("aGk="), Some(b"hi".to_vec()));
    assert_eq!(base64("aGk"), Some(b"hi".to_vec()));
    assert_eq!(base64("a+/Z"), Some(vec![0x6b, 0xef, 0xd9]));
    assert_eq!(base64("a b="), None);
    assert_eq!(base64("a"), None);
}
