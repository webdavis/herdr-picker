use super::*;

#[test]
fn a_chord_is_a_ctrl_or_alt_letter_then_any_plain_letters() {
    assert_eq!(
        Chord::parse("ctrl-t b").map(|chord| chord.strokes().to_vec()),
        Some(vec![Stroke::Ctrl('t'), Stroke::Plain('b')])
    );
    assert_eq!(
        Chord::parse("alt-g").map(|chord| chord.strokes().to_vec()),
        Some(vec![Stroke::Alt('g')])
    );
    assert_eq!(
        Chord::parse("ctrl-g  ctrl-x B").map(|chord| chord.strokes().to_vec()),
        Some(vec![
            Stroke::Ctrl('g'),
            Stroke::Ctrl('x'),
            Stroke::Plain('B')
        ])
    );
}

#[test]
fn ctrl_ignores_the_letter_case_the_way_a_terminal_does() {
    assert_eq!(Chord::parse("ctrl-T b"), Chord::parse("ctrl-t b"));
}

#[test]
fn a_chord_refuses_a_plain_start_an_unknown_modifier_or_a_non_letter() {
    for unusable in [
        "",
        "b",
        "b ctrl-t",
        "shift-t",
        "ctrl-tt",
        "ctrl-1",
        "ctrl-t 1",
        "ctrl-",
        "/",
        "ctrl-/",
        "ctrl-g '",
        "ctrl-g \"",
        "ctrl-g \\",
    ] {
        assert_eq!(Chord::parse(unusable), None, "{unusable:?}");
    }
}

#[test]
fn a_later_stroke_may_be_punctuation_that_keeps_a_readline_binding_whole() {
    assert_eq!(
        Chord::parse("ctrl-g /").map(|chord| chord.strokes().to_vec()),
        Some(vec![Stroke::Ctrl('g'), Stroke::Plain('/')])
    );
}
