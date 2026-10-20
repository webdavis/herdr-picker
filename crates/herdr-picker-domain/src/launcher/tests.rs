use super::*;
use crate::picker::{PickerCommand, PickerDeclaration, pickers};

fn declared(name: &str, key: &str) -> PickerDeclaration {
    PickerDeclaration {
        name: name.to_string(),
        key: key.to_string(),
        description: format!("{name} picker"),
        command: PickerCommand::Builtin(name.to_string()),
        args: Vec::new(),
    }
}

fn some_pickers() -> Vec<Picker> {
    pickers([
        declared("network", "ctrl-t n"),
        declared("branches", "ctrl-g b"),
        declared("files", "ctrl-t f"),
        declared("bookmark", "ctrl-t ctrl-b"),
        declared("contents", "ctrl-g /"),
        declared("quick", "alt-q"),
    ])
    .expect("pickers")
}

fn offered(base: &str) -> Vec<(char, String, String)> {
    let base = Chord::parse(base).expect("a chord").strokes()[0];
    launcher_targets(&some_pickers(), base)
        .into_iter()
        .map(|target| (target.pick_key, target.label, target.directory))
        .collect()
}

#[test]
fn the_launcher_offers_each_picker_on_its_base_by_the_key_after_it_in_key_order() {
    assert_eq!(
        offered("ctrl-t"),
        vec![
            ('f', "files".to_string(), "files picker".to_string()),
            ('n', "network".to_string(), "network picker".to_string()),
        ]
    );
    assert_eq!(
        offered("ctrl-g"),
        vec![
            ('/', "contents".to_string(), "contents picker".to_string()),
            ('b', "branches".to_string(), "branches picker".to_string()),
        ]
    );
}

#[test]
fn the_launcher_bases_are_each_first_key_a_picker_follows_with_one_more() {
    let names: Vec<String> = launcher_bases(&some_pickers())
        .into_iter()
        .map(|base| base.name())
        .collect();

    assert_eq!(names, vec!["ctrl-t".to_string(), "ctrl-g".to_string()]);
    assert_eq!(launcher_base("ctrl-g"), Some(Stroke::Ctrl('g')));
    assert_eq!(launcher_base("ctrl-g b"), None);
}

#[test]
fn a_typed_key_even_slash_starts_its_picker_and_enter_opens_the_search() {
    let browsing = |key| LauncherKeymap.command(key, Mode::Browsing);

    assert_eq!(browsing(PickKey::Character('b')), Command::Key('b'));
    assert_eq!(browsing(PickKey::Character('/')), Command::Key('/'));
    assert_eq!(browsing(PickKey::Enter), Command::Search);
    assert_eq!(browsing(PickKey::Escape), Command::Cancel);
    assert_eq!(
        LauncherKeymap.command(PickKey::Enter, Mode::Searching),
        Command::Accept
    );
}
