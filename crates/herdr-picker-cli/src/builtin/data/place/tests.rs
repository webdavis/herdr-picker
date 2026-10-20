use serde_json::json;

use super::*;

fn place(segments: &[Segment]) -> Place {
    Place(segments.to_vec())
}

fn key(name: &str) -> Segment {
    Segment::Key(name.to_string())
}

#[test]
fn a_place_is_labeled_as_the_jq_path_that_reaches_it() {
    let glob = place(&[key("workspace"), key("ignoreGlobs"), Segment::Index(0)]);

    assert_eq!(glob.label(), ".workspace.ignoreGlobs[0]");
    assert_eq!(glob.jq(), ".workspace.ignoreGlobs[0]");
    assert_eq!(place(&[]).label(), ".");
    assert_eq!(
        place(&[Segment::Index(3), key("message")]).label(),
        ".[3].message"
    );
    assert_eq!(
        place(&[key("odd key"), key("ok_1")]).label(),
        r#"."odd key".ok_1"#
    );
}

#[test]
fn a_decoded_place_is_labeled_by_its_keys_and_piped_through_the_decoding_in_jq() {
    let user = place(&[
        key("payload"),
        Segment::Decoded(Decoding::Base64),
        Segment::Decoded(Decoding::Json),
        key("user"),
    ]);

    assert_eq!(user.label(), ".payload.user");
    assert_eq!(user.jq(), ".payload | @base64d | fromjson | .user");
    assert!(user.decoded());
    assert_eq!(place(&[Segment::Decoded(Decoding::Json)]).jq(), "fromjson");
}

#[test]
fn a_shape_names_every_item_of_an_array_at_once() {
    let name = place(&[key("users"), Segment::Index(7), key("name")]);

    assert_eq!(name.shape(), ".users[].name");
}

#[test]
fn a_place_survives_its_state_form() {
    let user = place(&[
        key("payload"),
        Segment::Index(2),
        Segment::Decoded(Decoding::Base64),
    ]);

    assert_eq!(user.state(), r#"["payload",2,{"decoded":"base64"}]"#);
    assert_eq!(Place::parse(&user.state()), Some(user));
    assert_eq!(Place::parse("not json"), None);
}

#[test]
fn the_raw_part_of_a_place_is_its_path_in_the_file_before_any_decoding() {
    let user = place(&[
        key("payload"),
        Segment::Decoded(Decoding::Json),
        key("user"),
    ]);

    assert_eq!(user.raw(), vec![json!("payload")]);
}
