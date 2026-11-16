use super::*;

fn pass_keys_to() -> Vec<String> {
    vec!["nvim".to_string()]
}

#[test]
fn the_key_passes_on_over_neovim() {
    assert!(key_passes_on("shell", "nvim", &pass_keys_to()));
}

#[test]
fn the_key_passes_on_inside_pwes_own_float_whatever_runs_there() {
    assert!(key_passes_on(FLOAT_LABEL, "fzf", &pass_keys_to()));
}

#[test]
fn the_key_opens_pwe_over_a_shell_or_an_agent() {
    assert!(!key_passes_on("shell", "bash", &pass_keys_to()));
    assert!(!key_passes_on("shell", "claude", &pass_keys_to()));
}

#[test]
fn an_empty_pass_on_list_lets_pwe_open_over_neovim_too() {
    assert!(!key_passes_on("shell", "nvim", &[]));
}
