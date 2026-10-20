use super::*;

#[test]
fn the_mru_file_prefers_the_injected_directory_over_the_home_default() {
    assert_eq!(
        mru_file(Some("/tmp/state-override"), Some("/home/ignored")),
        PathBuf::from("/tmp/state-override/mru")
    );
    assert_eq!(
        mru_file(None, Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/mru")
    );
    assert_eq!(
        mru_file(Some(""), Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/mru")
    );
}

#[test]
fn the_spawned_jumps_log_sits_beside_the_mru_file() {
    assert_eq!(
        spawned_jump_log_file(Some("/tmp/state-override"), None),
        PathBuf::from("/tmp/state-override/jump.log")
    );
    assert_eq!(
        spawned_jump_log_file(None, Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/jump.log")
    );
}

#[test]
fn a_picker_run_caches_its_rows_in_a_file_of_its_own_beside_the_mru_file() {
    assert_eq!(
        picker_rows_file(Some("/tmp/state-override"), None, 4242),
        PathBuf::from("/tmp/state-override/picker-rows-4242.jsonl")
    );
    assert_eq!(
        picker_rows_file(None, Some("/home/me"), 7),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/picker-rows-7.jsonl")
    );
}

#[test]
fn a_popup_pickers_result_sits_in_the_state_directory_named_for_the_run() {
    assert_eq!(
        picker_result_file(None, Some("/home/me"), 42),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/picker-result-42")
    );
}

#[test]
fn the_bookmarks_file_sits_in_the_state_directory() {
    assert_eq!(
        bookmarks_file(None, Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/bookmarks")
    );
}

#[test]
fn the_quickfix_list_and_saved_checks_sit_in_the_state_folder() {
    assert_eq!(
        quickfix_file(Some("/tmp/state"), None),
        PathBuf::from("/tmp/state/quickfix.json")
    );
    assert_eq!(
        captures_dir(None, Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-picker/captures")
    );
}
