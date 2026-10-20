use super::*;

fn state(view: View) -> ViewState {
    ViewState {
        view,
        titles: vec!["Git Branches".to_string(), "Delete main?".to_string()],
        description: "Branches of this repository".to_string(),
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
        "list\tlocal\nBranches of this repository\nGit Branches\nDelete main?"
    );
    assert_eq!(
        encode_view_state(&ViewState {
            view: View::Chooser { group: None },
            titles: vec!["Two\nlines".to_string()],
            description: String::new(),
        }),
        "chooser\t\n\nTwo lines"
    );
}

#[test]
fn an_empty_or_unknown_view_file_reads_as_the_streamed_rows() {
    let empty = ViewState {
        view: View::Streamed { group: None },
        titles: Vec::new(),
        description: String::new(),
    };
    assert_eq!(parse_view_state(""), empty);
    assert_eq!(
        parse_view_state("sideways\tlocal").view,
        View::Streamed {
            group: Some("local".to_string())
        }
    );
}
