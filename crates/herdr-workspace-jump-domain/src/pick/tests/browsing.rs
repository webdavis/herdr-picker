use super::*;

#[test]
fn a_new_menu_shows_every_workspace_in_order_with_the_bar_on_the_first() {
    let workspaces = three_workspaces();

    assert_eq!(
        shown(&Menu::new(&workspaces).frame()),
        [">Ivy", "homelab", "dotfiles"]
    );
}

#[test]
fn next_and_previous_move_the_bar_and_stop_at_the_ends_and_enter_picks_its_row() {
    let workspaces = three_workspaces();
    let mut menu = Menu::new(&workspaces);

    assert_eq!(menu.decide(PickKey::Previous), Pick::Pending);
    assert_eq!(shown(&menu.frame())[0], ">Ivy");
    for _ in 0..4 {
        assert_eq!(menu.decide(PickKey::Next), Pick::Pending);
    }
    assert_eq!(shown(&menu.frame()), ["Ivy", "homelab", ">dotfiles"]);
    menu.decide(PickKey::Previous);
    assert_eq!(menu.decide(PickKey::Enter), Pick::Selected(&workspaces[1]));
}

#[test]
fn a_workspace_key_picks_that_workspace_wherever_the_bar_is() {
    let workspaces = three_workspaces();
    let mut menu = Menu::new(&workspaces);
    menu.decide(PickKey::Next);

    assert_eq!(
        menu.decide(PickKey::Character('f')),
        Pick::Selected(&workspaces[2])
    );
    assert_eq!(
        menu.decide(PickKey::Character('i')),
        Pick::Selected(&workspaces[0])
    );
}

#[test]
fn an_uppercase_press_does_not_pick_a_lowercase_key() {
    let workspaces = three_workspaces();

    assert_eq!(
        Menu::new(&workspaces).decide(PickKey::Character('I')),
        Pick::Unbound
    );
}

#[test]
fn escape_q_and_an_interrupt_all_cancel() {
    let workspaces = three_workspaces();

    for cancelling in [PickKey::Escape, PickKey::Interrupt, PickKey::Character('q')] {
        assert_eq!(Menu::new(&workspaces).decide(cancelling), Pick::Cancelled);
    }
}

#[test]
fn an_unbound_character_and_an_unreadable_key_do_nothing() {
    let workspaces = three_workspaces();

    for unbound in [PickKey::Character('z'), PickKey::Other] {
        assert_eq!(Menu::new(&workspaces).decide(unbound), Pick::Unbound);
    }
}

#[test]
fn a_workspace_picking_on_q_is_reached_rather_than_shadowed_by_cancel() {
    let workspaces = targets(vec![("queue", None)]);

    assert_eq!(
        Menu::new(&workspaces).decide(PickKey::Character('q')),
        Pick::Selected(&workspaces[0])
    );
    assert_eq!(
        Menu::new(&workspaces).decide(PickKey::Escape),
        Pick::Cancelled
    );
}
