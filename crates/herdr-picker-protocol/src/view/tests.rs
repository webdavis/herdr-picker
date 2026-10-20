use super::*;

#[test]
fn a_view_survives_the_round_trip_through_its_file() {
    for view in [
        View::Streamed,
        View::List { group: None },
        View::List {
            group: Some("remote branches".to_string()),
        },
        View::Chooser {
            group: Some("local".to_string()),
        },
    ] {
        assert_eq!(parse_view(&encode_view(&view)), view);
    }
}

#[test]
fn a_view_file_reads_as_one_line_of_mode_tab_group() {
    assert_eq!(
        encode_view(&View::List {
            group: Some("local".to_string())
        }),
        "list\tlocal"
    );
    assert_eq!(encode_view(&View::Chooser { group: None }), "chooser\t");
    assert_eq!(encode_view(&View::Streamed), "streamed\t");
}

#[test]
fn an_empty_or_unknown_view_file_reads_as_the_streamed_rows() {
    assert_eq!(parse_view(""), View::Streamed);
    assert_eq!(parse_view("sideways\tlocal"), View::Streamed);
}
