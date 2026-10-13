use super::*;

#[test]
fn read_config_refuses_a_look_switch_that_is_not_true_or_false_naming_the_line() {
    let (refusal, rendered) = refusal_for(
        "look-not-boolean",
        "[pick]\nleaders = \"yes\"\n[workspaces]\nnetpulse = \"/opt/netpulse\"\n",
    );

    assert!(
        matches!(refusal, ConfigError::Unparseable { line: Some(2), .. }),
        "{rendered}"
    );
    assert!(rendered.contains("boolean"), "{rendered}");
}

#[test]
fn read_config_refuses_a_theme_it_does_not_have_naming_the_ones_it_has() {
    let (refusal, rendered) = refusal_for(
        "theme-unknown",
        "[pick]\ntheme = \"gruvbox\"\n[workspaces]\nnetpulse = \"/opt/netpulse\"\n",
    );

    assert!(
        matches!(refusal, ConfigError::Unparseable { line: Some(2), .. }),
        "{rendered}"
    );
    assert!(rendered.contains("catppuccin-mocha"), "{rendered}");
}

#[test]
fn read_config_refuses_a_pick_key_it_does_not_know_naming_the_switches() {
    let (refusal, rendered) = refusal_for(
        "look-unknown",
        "[pick]\nrows = \"stripes\"\n[workspaces]\nnetpulse = \"/opt/netpulse\"\n",
    );

    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
    assert!(
        rendered.contains("leaders") && rendered.contains("stripes"),
        "{rendered}"
    );
}
