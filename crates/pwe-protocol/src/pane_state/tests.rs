use super::*;

#[test]
fn each_state_is_written_as_its_word_and_what_it_carries() {
    assert_eq!(
        encode_pane_state(&PaneState::Running { pid: 4242 }),
        "running\n4242"
    );
    assert_eq!(
        encode_pane_state(&PaneState::Finished {
            instruction: Some("cd\t/opt/beta".to_string())
        }),
        "finished\ncd\t/opt/beta"
    );
    assert_eq!(
        encode_pane_state(&PaneState::Finished { instruction: None }),
        "finished\n"
    );
    assert_eq!(
        encode_pane_state(&PaneState::Failed {
            reason: "no picker is named files".to_string()
        }),
        "failed\nno picker is named files"
    );
}

#[test]
fn every_state_reads_back_as_itself() {
    for state in [
        PaneState::Running { pid: 4242 },
        PaneState::Finished {
            instruction: Some("run\tprintf 'a\\nb'\nsecond line".to_string()),
        },
        PaneState::Finished { instruction: None },
        PaneState::Failed {
            reason: "picker files: line 2 is not a picker row".to_string(),
        },
    ] {
        assert_eq!(parse_pane_state(&encode_pane_state(&state)), Some(state));
    }
}

#[test]
fn a_state_file_in_no_known_shape_reads_as_none() {
    for body in ["", "running\nnot-a-pid", "paused\n", "finished"] {
        assert_eq!(parse_pane_state(body), None, "{body:?}");
    }
}
