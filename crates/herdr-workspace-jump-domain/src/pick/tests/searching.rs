use super::*;

#[test]
fn search_starts_an_empty_search_over_every_workspace_with_the_bar_on_the_first() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Next);

    assert_eq!(menu.apply(Command::Search), Pick::Pending);

    let frame = menu.frame();
    assert_eq!(menu.mode(), Mode::Searching);
    assert_eq!(search(&frame), Some(String::new()));
    assert_eq!(shown(&frame), [">Ivy", "homelab", "dotfiles"]);
}

#[test]
fn a_search_shows_the_workspaces_the_matcher_keeps_in_the_matchers_order() {
    let workspaces = three_workspaces();
    let mut menu = Menu::new(&workspaces, None, Reversed);
    menu.apply(Command::Search);

    assert_eq!(shown(&menu.frame()), [">dotfiles", "homelab", "Ivy"]);
    assert_eq!(menu.apply(Command::Accept), Pick::Selected(&workspaces[2]));
}

#[test]
fn typing_narrows_the_rows_and_puts_the_bar_back_on_the_first() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    menu.apply(Command::Next);

    type_text(&mut menu, "o");

    let frame = menu.frame();
    assert_eq!(shown(&frame), [">homelab", "dotfiles"]);
    assert_eq!(search(&frame), Some("o".to_string()));
    assert_eq!(frame.total, 3);
}

#[test]
fn a_search_nothing_matches_shows_no_rows_and_accept_picks_nothing() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    type_text(&mut menu, "zz");

    assert!(menu.frame().rows.is_empty());
    assert_eq!(menu.apply(Command::Accept), Pick::Pending);
}

#[test]
fn searching_an_unbound_key_is_ignored() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    type_text(&mut menu, "ho");

    assert_eq!(menu.apply(Command::Unbound), Pick::Pending);
    assert_eq!(menu.mode(), Mode::Searching);
    assert_eq!(search(&menu.frame()), Some("ho".to_string()));
}

#[test]
fn erase_deletes_the_last_letter_and_ends_the_search_once_it_is_empty() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    type_text(&mut menu, "ho");

    assert_eq!(menu.apply(Command::Erase), Pick::Pending);
    assert_eq!(search(&menu.frame()), Some("h".to_string()));
    menu.apply(Command::Erase);
    assert_eq!(search(&menu.frame()), Some(String::new()));

    assert_eq!(menu.apply(Command::Erase), Pick::Pending);
    assert_eq!(menu.mode(), Mode::Browsing);
    assert_eq!(menu.frame().search, None);
    assert_eq!(
        menu.apply(Command::Key('i')),
        Pick::Selected(&workspaces[0])
    );
}
