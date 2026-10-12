use super::*;

#[test]
fn escape_ctrl_c_and_ctrl_g_cancel_enter_accepts_and_the_list_keys_move_in_either_mode() {
    for mode in [Mode::Browsing, Mode::Searching] {
        for cancel in [
            PickKey::Escape,
            PickKey::Control('c'),
            PickKey::Control('g'),
        ] {
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
    for unbound in [
        PickKey::Control('u'),
        PickKey::Left,
        PickKey::Backspace,
        PickKey::Other,
    ] {
        assert_eq!(
            command(unbound, Mode::Browsing),
            Command::Unbound,
            "{unbound:?}"
        );
    }
}

#[test]
fn searching_the_editing_keys_are_herdrs_own_for_its_fields() {
    let table = [
        (PickKey::Character('/'), Command::Edit(Edit::Insert('/'))),
        (PickKey::Character('q'), Command::Edit(Edit::Insert('q'))),
        (PickKey::Backspace, Command::Erase),
        (PickKey::Control('h'), Command::Erase),
        (PickKey::Delete, Command::Edit(Edit::DeleteForward)),
        (PickKey::Control('d'), Command::Edit(Edit::DeleteForward)),
        (PickKey::Left, Command::Edit(Edit::Backward)),
        (PickKey::Control('b'), Command::Edit(Edit::Backward)),
        (PickKey::Right, Command::Edit(Edit::Forward)),
        (PickKey::Control('f'), Command::Edit(Edit::Forward)),
        (PickKey::Home, Command::Edit(Edit::Start)),
        (PickKey::Control('a'), Command::Edit(Edit::Start)),
        (PickKey::End, Command::Edit(Edit::End)),
        (PickKey::Control('e'), Command::Edit(Edit::End)),
        (PickKey::Alt('b'), Command::Edit(Edit::WordBackward)),
        (PickKey::Alt('f'), Command::Edit(Edit::WordForward)),
        (PickKey::Control('u'), Command::Edit(Edit::CutToStart)),
        (PickKey::Control('k'), Command::Edit(Edit::CutToEnd)),
        (PickKey::Control('w'), Command::Edit(Edit::CutWordBackward)),
        (PickKey::WordBackspace, Command::Edit(Edit::CutWordBackward)),
        (PickKey::Alt('d'), Command::Edit(Edit::CutWordForward)),
        (PickKey::Control('y'), Command::Edit(Edit::Paste)),
    ];
    for (key, meaning) in table {
        assert_eq!(command(key, Mode::Searching), meaning, "{key:?}");
    }
}

#[test]
fn searching_a_key_with_no_editing_meaning_is_unbound() {
    for unbound in [PickKey::Control('z'), PickKey::Alt('x'), PickKey::Other] {
        assert_eq!(
            command(unbound, Mode::Searching),
            Command::Unbound,
            "{unbound:?}"
        );
    }
}
