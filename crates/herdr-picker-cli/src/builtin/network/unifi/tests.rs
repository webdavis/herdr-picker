use serde_json::json;

use super::*;

#[test]
fn pages_are_read_until_the_total_is_reached() {
    let mut asked = Vec::new();
    let items = pages(|offset| {
        asked.push(offset);
        Ok(match offset {
            0 => json!({"data": [{"id": "a"}, {"id": "b"}], "limit": 2, "totalCount": 3}),
            _ => json!({"data": [{"id": "c"}], "limit": 2, "totalCount": 3}),
        })
    });

    assert_eq!(items.map(|items| items.len()), Ok(3));
    assert_eq!(asked, [0, 2]);
}

#[test]
fn a_short_page_ends_a_list_without_a_total_and_an_early_empty_page_is_refused() {
    let short = pages(|_| Ok(json!({"data": [{"id": "a"}], "limit": 200})));
    assert_eq!(short.map(|items| items.len()), Ok(1));

    let early = pages(|offset| {
        Ok(
            json!({"data": if offset == 0 { json!([{"id": "a"}]) } else { json!([]) }, "limit": 1, "totalCount": 5}),
        )
    });
    assert_eq!(
        early,
        Err("UniFi returned an empty early page; results are incomplete".to_string())
    );

    let malformed = pages(|_| Ok(json!({"error": "nope"})));
    assert_eq!(
        malformed,
        Err("Malformed UniFi page; results are incomplete".to_string())
    );
}

#[test]
fn without_settings_one_row_says_unifi_is_not_configured() {
    let (seen, notices) = unifi_sightings(None, None);

    assert!(seen.is_empty());
    assert_eq!(notices.len(), 1);
    assert_eq!(
        notices[0].label,
        "\u{1b}[2mUniFi is not configured\u{1b}[0m"
    );
}
