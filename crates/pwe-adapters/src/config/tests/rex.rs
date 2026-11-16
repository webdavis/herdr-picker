use super::*;

fn rex_settings(name: &str, content: &str) -> RexSettings {
    let sandbox = TempDirectory::new(name);
    read_config(&sandbox.write(content)).expect("a config").rex
}

#[test]
fn rex_passes_keys_to_neovim_and_looks_for_rex_beta_unless_told_otherwise() {
    assert_eq!(
        rex_settings("rex-default", ""),
        RexSettings {
            pass_keys_to: vec!["nvim".to_string()],
            app_name: "Rex Beta".to_string(),
        }
    );
}

#[test]
fn the_rex_table_names_the_programs_keys_pass_to_and_the_app() {
    assert_eq!(
        rex_settings(
            "rex-set",
            "[rex]\npass-keys-to = [\"nvim\", \"vim\"]\napp-name = \"Rex\"\n"
        ),
        RexSettings {
            pass_keys_to: vec!["nvim".to_string(), "vim".to_string()],
            app_name: "Rex".to_string(),
        }
    );
}
