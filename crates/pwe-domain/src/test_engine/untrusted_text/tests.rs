use super::*;

#[test]
fn control_characters_are_stripped_and_counted() {
    let text = UntrustedText::new("red \u{1b}[31mbold\u{7}\r");

    assert_eq!((text.text.as_str(), text.stripped), ("red [31mbold", 3));
}

#[test]
fn newlines_and_tabs_stay() {
    let text = UntrustedText::new("left:\t2\nright:\t3\n");

    assert_eq!(
        (text.text.as_str(), text.stripped),
        ("left:\t2\nright:\t3\n", 0)
    );
}
