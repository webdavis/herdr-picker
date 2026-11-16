use super::*;

const BLUEBUBBLES: &str = r#"
[pickers.apple.bluebubbles]
url = "https://messages.example.net:1234"
keychain-service = "BlueBubbles server password"
"#;

#[test]
fn the_apple_pickers_bluebubbles_table_names_the_server_and_the_keychain_item() {
    let sandbox = TempDirectory::new("bluebubbles");
    let config = read_config(&sandbox.write(BLUEBUBBLES)).expect("a readable config");

    assert_eq!(
        config.bluebubbles,
        Some(BlueBubblesSettings {
            url: "https://messages.example.net:1234".to_string(),
            keychain_service: "BlueBubbles server password".to_string(),
        })
    );
    assert!(config.pickers.iter().any(|picker| picker.name == "apple"));
}

#[test]
fn without_a_bluebubbles_table_bluebubbles_is_not_configured() {
    let sandbox = TempDirectory::new("no-bluebubbles");
    let config = read_config(&sandbox.write("")).expect("a readable config");

    assert_eq!(config.bluebubbles, None);
}

#[test]
fn a_bluebubbles_table_under_another_picker_is_refused() {
    let (refusal, rendered) = refusal_for(
        "misplaced-bluebubbles",
        &BLUEBUBBLES.replace("pickers.apple", "pickers.network"),
    );

    assert!(
        matches!(refusal, ConfigError::Misplaced { .. }),
        "{rendered}"
    );
    assert!(
        rendered.ends_with("[pickers.network.bluebubbles] belongs under [pickers.apple]"),
        "{rendered}"
    );
}
