use super::*;

fn field(text: &str, cursor: usize) -> ReadlineField {
    let mut field = ReadlineField::default();
    for character in text.chars() {
        field.apply(Edit::Insert(character));
    }
    while field.cursor() > cursor {
        field.apply(Edit::Backward);
    }
    field
}

fn shown(field: &ReadlineField) -> String {
    let mut text = field.text();
    let byte = text
        .char_indices()
        .nth(field.cursor())
        .map_or(text.len(), |(byte, _)| byte);
    text.insert(byte, '|');
    text
}

fn after(text: &str, cursor: usize, edits: &[Edit]) -> String {
    let mut field = field(text, cursor);
    for &edit in edits {
        field.apply(edit);
    }
    shown(&field)
}

#[test]
fn typing_inserts_at_the_cursor() {
    assert_eq!(after("", 0, &[]), "|");
    assert_eq!(after("homelab", 4, &[Edit::Insert('X')]), "homeX|lab");
    assert!(ReadlineField::default().is_empty());
}

#[test]
fn the_cursor_counts_characters_not_bytes() {
    let mut field = ReadlineField::default();

    field.apply(Edit::Insert('é'));

    assert_eq!(field.cursor(), 1);
}

#[test]
fn the_cursor_moves_by_character_and_stops_at_both_ends() {
    assert_eq!(after("ivy", 1, &[Edit::Backward]), "|ivy");
    assert_eq!(after("ivy", 0, &[Edit::Backward]), "|ivy");
    assert_eq!(after("ivy", 2, &[Edit::Forward]), "ivy|");
    assert_eq!(after("ivy", 3, &[Edit::Forward]), "ivy|");
}

#[test]
fn start_and_end_jump_to_the_ends() {
    assert_eq!(after("homelab", 3, &[Edit::Start]), "|homelab");
    assert_eq!(after("homelab", 3, &[Edit::End]), "homelab|");
}

#[test]
fn the_cursor_moves_by_word_over_dashes() {
    assert_eq!(
        after("feed-case-study", 15, &[Edit::WordBackward]),
        "feed-case-|study"
    );
    assert_eq!(
        after("feed-case-study", 10, &[Edit::WordBackward]),
        "feed-|case-study"
    );
    assert_eq!(
        after("feed-case-study", 0, &[Edit::WordForward]),
        "feed|-case-study"
    );
    assert_eq!(
        after("feed-case-study", 4, &[Edit::WordForward]),
        "feed-case|-study"
    );
    assert_eq!(after("feed", 0, &[Edit::WordBackward]), "|feed");
    assert_eq!(after("feed", 4, &[Edit::WordForward]), "feed|");
}

#[test]
fn delete_removes_one_character_on_either_side_and_nothing_past_an_end() {
    assert_eq!(after("ivy", 2, &[Edit::DeleteBackward]), "i|y");
    assert_eq!(after("ivy", 0, &[Edit::DeleteBackward]), "|ivy");
    assert_eq!(after("ivy", 1, &[Edit::DeleteForward]), "i|y");
    assert_eq!(after("ivy", 3, &[Edit::DeleteForward]), "ivy|");
}

#[test]
fn cutting_to_either_end_removes_that_side_of_the_cursor() {
    assert_eq!(after("homelab", 4, &[Edit::CutToStart]), "|lab");
    assert_eq!(after("homelab", 4, &[Edit::CutToEnd]), "home|");
}

#[test]
fn cutting_a_word_removes_the_word_on_that_side() {
    assert_eq!(after("feed-case", 9, &[Edit::CutWordBackward]), "feed-|");
    assert_eq!(after("feed-case", 0, &[Edit::CutWordForward]), "|-case");
}

#[test]
fn paste_inserts_the_last_cut_text_and_cutting_nothing_keeps_it() {
    assert_eq!(
        after("homelab", 4, &[Edit::CutToEnd, Edit::Start, Edit::Paste]),
        "lab|home"
    );
    assert_eq!(
        after(
            "homelab",
            4,
            &[Edit::CutToStart, Edit::CutToStart, Edit::End, Edit::Paste]
        ),
        "labhome|"
    );
    assert_eq!(after("ivy", 1, &[Edit::Paste]), "i|vy");
}
