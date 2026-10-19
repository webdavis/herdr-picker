use super::*;

#[test]
fn search_starts_an_empty_search_over_every_workspace_with_the_bar_on_the_first() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Next);

    assert_eq!(menu.apply(Command::Search), Pick::Pending);

    let frame = menu.frame();
    assert_eq!(menu.mode(), Mode::Searching);
    assert_eq!(search(&frame), Some((String::new(), 0)));
    assert_eq!(shown(&frame), [">Ivy", "homelab", "dotfiles"]);
}

#[test]
fn a_search_shows_the_workspaces_the_matcher_keeps_in_the_matchers_order() {
    let workspaces = three_workspaces();
    let mut menu = Menu::new(&workspaces, &[], None, Reversed, ReadlineField::default());
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
    assert_eq!(search(&frame), Some(("o".to_string(), 1)));
    assert_eq!(frame.total, 3);
}

#[test]
fn a_search_lights_the_letters_the_matcher_found_and_browsing_lights_none() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    type_text(&mut menu, "ot");

    let frame = menu.frame();
    assert_eq!(shown(&frame), [">dotfiles"]);
    assert_eq!(frame.rows[0].matched, [1, 2]);

    for _ in 0..3 {
        menu.apply(Command::Erase);
    }
    assert_eq!(menu.mode(), Mode::Browsing);
    assert!(menu.frame().rows.iter().all(|row| row.matched.is_empty()));
}

#[test]
fn a_search_never_offers_the_workspace_the_popup_opened_from() {
    let workspaces = three_workspaces();
    let live = herdr_has(&["homelab"]);
    let mut menu = Menu::new(
        &workspaces,
        &live,
        Some("w_homelab"),
        Contains,
        ReadlineField::default(),
    );
    menu.apply(Command::Search);

    type_text(&mut menu, "o");

    let frame = menu.frame();
    assert_eq!(shown(&frame), [">dotfiles"]);
    assert_eq!(frame.total, 2);
}

#[test]
fn moving_within_the_search_leaves_the_bar_on_its_row() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    type_text(&mut menu, "o");
    menu.apply(Command::Next);

    assert_eq!(menu.apply(Command::Edit(Edit::Start)), Pick::Pending);

    let frame = menu.frame();
    assert_eq!(shown(&frame), ["homelab", ">dotfiles"]);
    assert_eq!(search(&frame), Some(("o".to_string(), 0)));
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
    assert_eq!(search(&menu.frame()), Some(("ho".to_string(), 2)));
}

#[test]
fn erase_deletes_behind_the_cursor_and_ends_the_search_once_it_is_empty() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Search);
    type_text(&mut menu, "ho");

    menu.apply(Command::Edit(Edit::Start));
    assert_eq!(menu.apply(Command::Erase), Pick::Pending);
    assert_eq!(
        menu.mode(),
        Mode::Searching,
        "text after the cursor keeps the search"
    );

    menu.apply(Command::Edit(Edit::End));
    menu.apply(Command::Erase);
    menu.apply(Command::Erase);
    assert_eq!(search(&menu.frame()), Some((String::new(), 0)));

    assert_eq!(menu.apply(Command::Erase), Pick::Pending);
    assert_eq!(menu.mode(), Mode::Browsing);
    assert_eq!(menu.frame().search, None);
    assert_eq!(
        menu.apply(Command::Key('i')),
        Pick::Selected(&workspaces[0])
    );
}

#[test]
fn the_menu_hands_each_edit_to_its_field_and_shows_what_the_field_holds() {
    let workspaces = three_workspaces();
    let mut menu = Menu::new(&workspaces, &[], None, Contains, RecordingField::default());
    menu.apply(Command::Search);

    menu.apply(Command::Edit(Edit::Insert('h')));
    menu.apply(Command::Edit(Edit::CutToStart));
    menu.apply(Command::Erase);

    assert_eq!(
        menu.field.edits,
        [Edit::Insert('h'), Edit::CutToStart, Edit::DeleteBackward]
    );
    let frame = menu.frame();
    assert_eq!(search(&frame), Some(("h".to_string(), 1)));
    assert_eq!(shown(&frame), [">homelab"]);
}
