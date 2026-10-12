use super::*;
use crate::{WorkspaceDeclaration, jump_targets};

fn targets(declarations: Vec<(&str, Option<&str>)>) -> Vec<JumpTarget> {
    let declared = declarations
        .into_iter()
        .map(|(label, key)| WorkspaceDeclaration {
            label: label.to_string(),
            directory: format!("/opt/{label}"),
            key: key.map(str::to_string),
        })
        .collect::<Vec<_>>();
    jump_targets(declared).expect("well formed workspaces")
}

fn three_workspaces() -> Vec<JumpTarget> {
    targets(vec![
        ("Ivy", None),
        ("homelab", None),
        ("dotfiles", Some("f")),
    ])
}

#[test]
fn the_menu_lists_each_workspace_with_the_first_on_a_grey_bar_and_ends_with_the_key_legend() {
    let workspaces = three_workspaces();

    assert_eq!(
        Menu::new(&workspaces).render(20),
        "\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy           \x1b[0m\n \
         \x1b[1;34m  h\x1b[0m  homelab\n \
         \x1b[1;34m  f\x1b[0m  dotfiles\n\
         \n\
         \x1b[2m ctrl+n/p move  enter jump  esc close\x1b[0m"
    );
}

#[test]
fn a_width_too_narrow_for_the_bar_pads_nothing_and_cuts_nothing() {
    let workspaces = three_workspaces();

    assert_eq!(
        Menu::new(&workspaces).render(0).lines().next(),
        Some("\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy\x1b[0m")
    );
}

#[test]
fn next_and_previous_move_the_bar_and_stop_at_the_ends_and_enter_picks_its_row() {
    let workspaces = three_workspaces();
    let mut menu = Menu::new(&workspaces);

    assert_eq!(menu.decide(PickKey::Previous), Pick::Changed);
    assert_eq!(menu.decide(PickKey::Enter), Pick::Selected(&workspaces[0]));
    for _ in 0..4 {
        assert_eq!(menu.decide(PickKey::Next), Pick::Changed);
    }
    assert_eq!(menu.decide(PickKey::Enter), Pick::Selected(&workspaces[2]));
    assert_eq!(menu.decide(PickKey::Previous), Pick::Changed);
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
