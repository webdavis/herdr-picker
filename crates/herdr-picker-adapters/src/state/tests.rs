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
