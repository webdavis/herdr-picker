use super::*;
use pwe_application::DesktopRoute;
use pwe_domain::{Corner, InboxLimits, NoticeSettings};

fn read(name: &str, content: &str) -> PluginConfig {
    let sandbox = TempDirectory::new(name);
    read_config(&sandbox.write(content)).expect("a config")
}

#[test]
fn notices_and_the_inbox_take_the_defaults_unless_told_otherwise() {
    let config = read("notices-default", "");

    assert_eq!(
        config.notices,
        NoticeConfig {
            corner: Corner::TopRight,
            board: NoticeSettings::default(),
            macos: DesktopRoute::Auto,
        }
    );
    assert_eq!(config.inbox, InboxLimits::default());
}

#[test]
fn the_notices_and_inbox_tables_set_every_key() {
    let config = read(
        "notices-set",
        "[notices]\ncorner = \"bottom-left\"\nfinished-seconds = 4\nneeds-you-seconds = 12\nmost-on-screen = 2\nseen-after-seconds = 3\nask-session-every-seconds = 5\nmacos = \"terminal-notifier\"\n\n[inbox]\nkeep-days = 7\nmost-rows = 100\n",
    );

    assert_eq!(
        config.notices,
        NoticeConfig {
            corner: Corner::BottomLeft,
            board: NoticeSettings {
                finished_seconds: 4,
                needs_you_seconds: 12,
                most_on_screen: 2,
                seen_after_seconds: 3,
                ask_session_every_seconds: 5,
            },
            macos: DesktopRoute::TerminalNotifier,
        }
    );
    assert_eq!(
        config.inbox,
        InboxLimits {
            keep_days: 7,
            most_rows: 100,
        }
    );
}

#[test]
fn an_unknown_corner_is_refused_naming_the_table_the_key_and_the_accepted_words() {
    let (refusal, rendered) = refusal_for("corner", "[notices]\ncorner = \"middle\"\n");

    assert!(
        matches!(refusal, ConfigError::UnknownWord { .. }),
        "{rendered}"
    );
    for part in [
        "[notices]",
        "corner",
        "\"middle\"",
        "top-right, top-left, bottom-right, bottom-left",
    ] {
        assert!(rendered.contains(part), "{part} missing from {rendered}");
    }
}

#[test]
fn an_unknown_macos_route_is_refused_naming_the_accepted_words() {
    let (_, rendered) = refusal_for("macos", "[notices]\nmacos = \"growl\"\n");

    assert!(
        rendered.contains("[notices] macos")
            && rendered.contains("auto, osascript, terminal-notifier, off"),
        "{rendered}"
    );
}

#[test]
fn an_unknown_key_in_the_inbox_table_is_refused() {
    let (_, rendered) = refusal_for("inbox-key", "[inbox]\nkeep-weeks = 2\n");

    assert!(rendered.contains("unknown field"), "{rendered}");
}
