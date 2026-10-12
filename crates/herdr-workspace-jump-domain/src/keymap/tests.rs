use super::*;

#[test]
fn escape_and_ctrl_c_cancel_enter_accepts_and_the_list_keys_move_in_either_mode() {
    for mode in [Mode::Browsing, Mode::Searching] {
        for cancel in [PickKey::Escape, PickKey::Control('c')] {
            assert_eq!(command(cancel, mode), Command::Cancel, "{cancel:?}");
        }
        assert_eq!(command(PickKey::Enter, mode), Command::Accept);
        for next in [PickKey::Down, PickKey::Control('n')] {
            assert_eq!(command(next, mode), Command::Next, "{next:?}");
        }
        for previous in [PickKey::Up, PickKey::Control('p')] {
            assert_eq!(command(previous, mode), Command::Previous, "{previous:?}");
        }
    }
}

#[test]
fn browsing_slash_searches_a_character_is_a_key_and_anything_else_is_unbound() {
    assert_eq!(
        command(PickKey::Character('/'), Mode::Browsing),
        Command::Search
    );
    assert_eq!(
        command(PickKey::Character('h'), Mode::Browsing),
        Command::Key('h')
    );
    for unbound in [PickKey::Control('u'), PickKey::Backspace, PickKey::Other] {
        assert_eq!(
            command(unbound, Mode::Browsing),
            Command::Unbound,
            "{unbound:?}"
        );
    }
}

#[test]
fn searching_a_character_types_it_and_backspace_erases() {
    for character in ['/', 'q', 'f'] {
        assert_eq!(
            command(PickKey::Character(character), Mode::Searching),
            Command::Edit(Edit::Insert(character)),
            "{character:?}"
        );
    }
    assert_eq!(command(PickKey::Backspace, Mode::Searching), Command::Erase);
}

#[test]
fn searching_a_key_with_no_editing_meaning_is_unbound() {
    for unbound in [PickKey::Control('z'), PickKey::Other] {
        assert_eq!(
            command(unbound, Mode::Searching),
            Command::Unbound,
            "{unbound:?}"
        );
    }
}
