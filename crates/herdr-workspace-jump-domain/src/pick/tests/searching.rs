use super::*;

#[test]
fn slash_starts_an_empty_search_over_every_workspace_with_the_bar_on_the_first() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Next);

    assert_eq!(menu.decide(PickKey::Character('/')), Pick::Pending);

    let frame = menu.frame();
    assert_eq!(search(&frame), Some(String::new()));
    assert_eq!(shown(&frame), [">Ivy", "homelab", "dotfiles"]);
}

#[test]
fn typing_narrows_the_rows_and_puts_the_bar_back_on_the_first() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Character('/'));
    menu.decide(PickKey::Next);

    type_text(&mut menu, "o");

    let frame = menu.frame();
    assert_eq!(shown(&frame), [">homelab", "dotfiles"]);
    assert_eq!(search(&frame), Some("o".to_string()));
    assert_eq!(frame.total, 3);
}

#[test]
fn a_label_matches_when_it_holds_the_letters_in_order_ignoring_case() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Character('/'));

    type_text(&mut menu, "HLB");

    assert_eq!(shown(&menu.frame()), [">homelab"]);
}

#[test]
fn a_search_nothing_matches_shows_no_rows_and_enter_picks_nothing() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Character('/'));
    type_text(&mut menu, "zz");

    assert!(menu.frame().rows.is_empty());
    assert_eq!(menu.decide(PickKey::Enter), Pick::Pending);
}

#[test]
fn searching_a_workspace_key_q_and_a_slash_are_letters_of_the_search() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Character('/'));

    type_text(&mut menu, "fq/");

    assert_eq!(search(&menu.frame()), Some("fq/".to_string()));
}

#[test]
fn searching_an_unreadable_key_is_ignored() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Character('/'));
    type_text(&mut menu, "ho");

    assert_eq!(menu.decide(PickKey::Other), Pick::Pending);
    assert_eq!(search(&menu.frame()), Some("ho".to_string()));
}

#[test]
fn backspace_deletes_the_last_letter_and_ends_the_search_once_it_is_empty() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.decide(PickKey::Character('/'));
    type_text(&mut menu, "ho");

    assert_eq!(menu.decide(PickKey::Backspace), Pick::Pending);
    assert_eq!(search(&menu.frame()), Some("h".to_string()));
    menu.decide(PickKey::Backspace);
    assert_eq!(search(&menu.frame()), Some(String::new()));

    assert_eq!(menu.decide(PickKey::Backspace), Pick::Pending);
    assert_eq!(menu.frame().search, None);
    assert_eq!(
        menu.decide(PickKey::Character('i')),
        Pick::Selected(&workspaces[0])
    );
}

#[test]
fn escape_and_an_interrupt_close_a_search_too() {
    let workspaces = three_workspaces();

    for cancelling in [PickKey::Escape, PickKey::Interrupt] {
        let mut menu = menu(&workspaces);
        menu.decide(PickKey::Character('/'));
        assert_eq!(menu.decide(cancelling), Pick::Cancelled, "{cancelling:?}");
    }
}
