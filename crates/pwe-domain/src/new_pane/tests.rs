use super::*;

#[test]
fn alt_h_j_k_l_open_left_below_above_right_and_alt_n_a_new_tab() {
    let opened: Vec<Option<NewPane>> = ["alt-h", "alt-j", "alt-k", "alt-l", "alt-n", "alt-m"]
        .iter()
        .map(|name| NewPane::of(&Key::parse(name).expect("a key")))
        .collect();

    assert_eq!(
        opened,
        vec![
            Some(NewPane::Left),
            Some(NewPane::Below),
            Some(NewPane::Above),
            Some(NewPane::Right),
            Some(NewPane::Tab),
            None
        ]
    );
}

#[test]
fn each_new_pane_round_trips_through_its_word() {
    let words: Vec<&str> = [
        NewPane::Left,
        NewPane::Below,
        NewPane::Above,
        NewPane::Right,
        NewPane::Tab,
    ]
    .map(NewPane::word)
    .to_vec();

    assert_eq!(words, vec!["left", "below", "above", "right", "tab"]);
    assert_eq!(NewPane::parse("above"), Some(NewPane::Above));
    assert_eq!(NewPane::parse("run"), None);
}
