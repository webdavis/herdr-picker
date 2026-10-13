use super::*;

#[test]
fn a_new_menu_browses_every_workspace_in_order_with_the_bar_on_the_first() {
    let workspaces = three_workspaces();
    let menu = menu(&workspaces);
    let frame = menu.frame();

    assert_eq!(menu.mode(), Mode::Browsing);
    assert_eq!(frame.search, None);
    assert_eq!(shown(&frame), [">Ivy", "homelab", "dotfiles"]);
    assert_eq!(frame.total, 3);
}

#[test]
fn the_workspace_the_popup_opened_from_is_left_out_and_an_unknown_one_leaves_the_list_whole() {
    let workspaces = three_workspaces();

    let live = herdr_has(&["homelab"]);
    let opened = Menu::new(
        &workspaces,
        &live,
        Some("w_homelab"),
        Contains,
        ReadlineField::default(),
    );
    assert_eq!(shown(&opened.frame()), [">Ivy", "dotfiles"]);
    assert_eq!(opened.frame().total, 2);

    let live = herdr_has(&["elsewhere"]);
    let unknown = Menu::new(
        &workspaces,
        &live,
        Some("w_elsewhere"),
        Contains,
        ReadlineField::default(),
    );
    assert_eq!(shown(&unknown.frame()), [">Ivy", "homelab", "dotfiles"]);
    assert_eq!(unknown.frame().total, 3);
}

#[test]
fn the_key_of_the_workspace_the_popup_opened_from_keeps_the_menu_open_and_says_so_until_the_next_command()
 {
    let workspaces = three_workspaces();
    let live = herdr_has(&["homelab"]);
    let mut menu = Menu::new(
        &workspaces,
        &live,
        Some("w_homelab"),
        Contains,
        ReadlineField::default(),
    );

    assert_eq!(menu.apply(Command::Key('h')), Pick::Pending);

    let frame = menu.frame();
    assert_eq!(frame.refusal, Some(Refusal::AlreadyCurrent(&workspaces[1])));
    assert_eq!(shown(&frame), [">Ivy", "dotfiles"]);
    assert_eq!(menu.apply(Command::Next), Pick::Pending);
    assert_eq!(menu.frame().refusal, None);
}

#[test]
fn a_workspace_herdr_has_open_is_marked_open_in_its_row_and_the_others_are_not() {
    let workspaces = three_workspaces();
    let live = herdr_has(&["dotfiles", "elsewhere"]);
    let menu = Menu::new(&workspaces, &live, None, Contains, ReadlineField::default());

    let open: Vec<bool> = menu.frame().rows.iter().map(|row| row.open).collect();

    assert_eq!(open, [false, false, true]);
}

#[test]
fn an_active_id_herdr_does_not_list_leaves_every_workspace_offered() {
    let workspaces = three_workspaces();
    let live = herdr_has(&["homelab"]);
    let menu = Menu::new(
        &workspaces,
        &live,
        Some("w_gone"),
        Contains,
        ReadlineField::default(),
    );

    assert_eq!(menu.frame().total, 3);
}

#[test]
fn next_and_previous_move_the_bar_and_stop_at_the_ends_and_accept_picks_its_row() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);

    assert_eq!(menu.apply(Command::Previous), Pick::Pending);
    assert_eq!(shown(&menu.frame())[0], ">Ivy");
    for _ in 0..4 {
        assert_eq!(menu.apply(Command::Next), Pick::Pending);
    }
    assert_eq!(shown(&menu.frame()), ["Ivy", "homelab", ">dotfiles"]);
    menu.apply(Command::Previous);
    assert_eq!(menu.apply(Command::Accept), Pick::Selected(&workspaces[1]));
}

#[test]
fn a_workspace_key_picks_that_workspace_wherever_the_bar_is() {
    let workspaces = three_workspaces();
    let mut menu = menu(&workspaces);
    menu.apply(Command::Next);

    assert_eq!(
        menu.apply(Command::Key('f')),
        Pick::Selected(&workspaces[2])
    );
    assert_eq!(
        menu.apply(Command::Key('i')),
        Pick::Selected(&workspaces[0])
    );
}

#[test]
fn the_key_that_picks_is_the_exact_declared_character() {
    let workspaces = three_workspaces();

    assert_eq!(menu(&workspaces).apply(Command::Key('I')), Pick::Unbound);
}

#[test]
fn cancel_closes_and_a_key_no_workspace_claims_closes_unless_a_workspace_claims_it() {
    let workspaces = three_workspaces();
    assert_eq!(menu(&workspaces).apply(Command::Cancel), Pick::Cancelled);
    assert_eq!(menu(&workspaces).apply(Command::Key('q')), Pick::Unbound);

    let queue = targets(vec![("queue", None)]);
    assert_eq!(
        menu(&queue).apply(Command::Key('q')),
        Pick::Selected(&queue[0])
    );
}

#[test]
fn browsing_a_command_only_a_search_can_use_closes_the_menu() {
    let workspaces = three_workspaces();

    for unbound in [
        Command::Unbound,
        Command::Erase,
        Command::Edit(Edit::Insert('x')),
    ] {
        assert_eq!(
            menu(&workspaces).apply(unbound),
            Pick::Unbound,
            "{unbound:?}"
        );
    }
}
