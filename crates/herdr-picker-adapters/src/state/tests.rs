use super::*;

#[test]
fn the_state_folder_is_the_one_herdr_names_then_xdg_state_home_then_the_home_default() {
    assert_eq!(
        plugin_state_dir(Some("/tmp/herdr-named"), Some("/xdg"), Some("/home/me")),
        PathBuf::from("/tmp/herdr-named")
    );
    assert_eq!(
        plugin_state_dir(None, Some("/xdg"), Some("/home/me")),
        PathBuf::from("/xdg/herdr/plugins/herdr-picker")
    );
    assert_eq!(
        plugin_state_dir(Some(""), Some(""), Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker")
    );
}

#[test]
fn each_state_file_sits_in_the_state_folder() {
    let state = Path::new("/state");

    assert_eq!(
        [
            mru_file(state),
            spawned_jump_log_file(state),
            places_file(state),
            snapshot_file(state),
            send_queue_file(state),
            bookmarks_file(state),
            quickfix_file(state),
            captures_dir(state),
            picker_rows_file(state, 7),
            picker_result_file(state, 42),
        ],
        [
            "/state/mru",
            "/state/jump.log",
            "/state/places",
            "/state/agents.json",
            "/state/send-queue.json",
            "/state/bookmarks",
            "/state/quickfix.json",
            "/state/captures",
            "/state/picker-rows-7.jsonl",
            "/state/picker-result-42",
        ]
        .map(PathBuf::from)
    );
}
