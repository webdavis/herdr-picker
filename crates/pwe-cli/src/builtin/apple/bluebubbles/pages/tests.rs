use serde_json::json;

use super::*;

#[test]
fn pages_are_read_until_the_total_each_record_once() {
    let mut asked = Vec::new();
    let read: Vec<_> = pages(|offset| {
        asked.push(offset);
        Ok(match offset {
            0 => json!({"data": [{"guid": "a"}, {"guid": "b"}], "metadata": {"total": 3}}),
            _ => json!({"data": [{"guid": "b"}, {"guid": "c"}], "metadata": {"total": 3}}),
        })
    })
    .collect();

    assert_eq!(
        read,
        [
            Ok(json!({"guid": "a"})),
            Ok(json!({"guid": "b"})),
            Ok(json!({"guid": "c"}))
        ]
    );
    assert_eq!(asked, [0, 2]);
}

#[test]
fn a_page_that_brings_nothing_new_or_a_record_without_a_guid_ends_the_list_saying_so() {
    let stuck: Vec<_> = pages(|_| Ok(json!({"data": [{"guid": "a"}]}))).collect();
    assert_eq!(
        stuck,
        [
            Ok(json!({"guid": "a"})),
            Err("BlueBubbles pagination did not advance; results are incomplete.".to_string())
        ]
    );

    let anonymous: Vec<_> = pages(|_| Ok(json!({"data": [{"text": "hi"}]}))).collect();
    assert_eq!(
        anonymous,
        [Err(
            "BlueBubbles returned a record without a stable identifier.".to_string()
        )]
    );
}
