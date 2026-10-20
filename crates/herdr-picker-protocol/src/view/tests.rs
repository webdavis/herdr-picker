use super::*;

fn state(view: View) -> ViewState {
    ViewState {
        view,
        titles: vec!["Git Branches".to_string(), "Delete main?".to_string()],
        description: "Branches of this repository".to_string(),
        source: None,
        sources: Vec::new(),
    }
}

#[test]
fn a_view_state_survives_the_round_trip_through_its_file() {
    for view in [
        View::Streamed { group: None },
        View::Streamed {
            group: Some("local".to_string()),
        },
        View::List { group: None },
        View::List {
            group: Some("remote branches".to_string()),
        },
        View::Chooser {
            group: Some("local".to_string()),
        },
    ] {
        assert_eq!(
            parse_view_state(&encode_view_state(&state(view.clone()))),
            state(view)
        );
    }
}

#[test]
fn a_view_file_reads_as_mode_tab_group_then_the_description_then_one_title_a_line() {
    assert_eq!(
        encode_view_state(&state(View::List {
            group: Some("local".to_string())
        })),
        "list\tlocal\t\nBranches of this repository\n\nGit Branches\nDelete main?"
    );
    assert_eq!(
        encode_view_state(&ViewState {
            view: View::Chooser { group: None },
            titles: vec!["Two\nlines".to_string()],
            description: String::new(),
            source: None,
            sources: Vec::new(),
        }),
        "chooser\t\t\n\n\nTwo lines"
    );
}

#[test]
fn an_empty_or_unknown_view_file_reads_as_the_streamed_rows() {
    let empty = ViewState {
        view: View::Streamed { group: None },
        titles: Vec::new(),
        description: String::new(),
        source: None,
        sources: Vec::new(),
    };
    assert_eq!(parse_view_state(""), empty);
    assert_eq!(
        parse_view_state("sideways\tlocal").view,
        View::Streamed {
            group: Some("local".to_string())
        }
    );
}

#[test]
fn a_view_file_keeps_the_source_on_the_first_line_and_the_sources_on_the_third() {
    let state = ViewState {
        view: View::Streamed { group: None },
        titles: vec!["Files".to_string()],
        description: String::new(),
        source: Some("whole disk".to_string()),
        sources: vec!["this folder".to_string(), "whole disk".to_string()],
    };

    let encoded = encode_view_state(&state);

    assert_eq!(
        encoded,
        "streamed\t\twhole disk\n\nthis folder\twhole disk\nFiles"
    );
    assert_eq!(parse_view_state(&encoded), state);
}
