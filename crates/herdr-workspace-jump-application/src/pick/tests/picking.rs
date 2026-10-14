use super::*;

#[test]
fn a_workspaces_key_picks_it_after_one_frame() {
    let targets = workspaces();
    let mut screen = Notes::default();

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Inputs::typing(&typed("d")),
        &mut screen,
    );

    assert_eq!(picked, Ok(Some(&targets[3])));
    assert_eq!(
        screen.frames,
        [">netpulse plantpulse essential dotfiles fitness"]
    );
}

#[test]
fn the_bar_moves_frame_by_frame_and_enter_picks_the_row_under_it() {
    let targets = workspaces();
    let mut screen = Notes::default();

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Inputs::typing(&[PickKey::Control('n'), PickKey::Control('n'), PickKey::Enter]),
        &mut screen,
    );

    assert_eq!(picked, Ok(Some(&targets[2])));
    assert_eq!(
        screen.frames,
        [
            ">netpulse plantpulse essential dotfiles fitness",
            "netpulse >plantpulse essential dotfiles fitness",
            "netpulse plantpulse >essential dotfiles fitness",
        ]
    );
}

#[test]
fn a_search_is_answered_frame_by_frame_and_enter_picks_the_best_match() {
    let targets = workspaces();
    let mut screen = Notes::default();
    let mut keys = typed("/es");
    keys.push(PickKey::Enter);

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Inputs::typing(&keys),
        &mut screen,
    );

    assert_eq!(picked, Ok(Some(&targets[2])));
    assert_eq!(
        screen.frames,
        [
            ">netpulse plantpulse essential dotfiles fitness",
            "/ >netpulse plantpulse essential dotfiles fitness",
            "/e >essential netpulse fitness dotfiles plantpulse",
            "/es >essential fitness dotfiles netpulse",
        ]
    );
}

#[test]
fn readline_keys_edit_the_search_between_frames() {
    let targets = workspaces();
    let mut screen = Notes::default();
    let mut keys = typed("/net");
    keys.extend([PickKey::Control('u')]);
    keys.extend(typed("fit"));
    keys.push(PickKey::Enter);

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Inputs::typing(&keys),
        &mut screen,
    );

    assert_eq!(picked, Ok(Some(&targets[4])));
    assert_eq!(
        screen.frames[5], "/ >netpulse plantpulse essential dotfiles fitness",
        "ctrl+u left only the slash"
    );
}

#[test]
fn escape_closes_with_nothing_picked() {
    let targets = workspaces();
    let mut screen = Notes::default();

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Inputs::typing(&[PickKey::Escape]),
        &mut screen,
    );

    assert_eq!(picked, Ok(None));
    assert_eq!(screen.frames.len(), 1);
}

#[test]
fn a_key_no_workspace_claims_closes_with_nothing_picked() {
    let targets = workspaces();

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Inputs::typing(&typed("z")),
        &mut Notes::default(),
    );

    assert_eq!(picked, Ok(None));
}

#[test]
fn the_keymap_is_asked_in_the_mode_the_menu_is_in() {
    let targets = workspaces();
    let keymap = ModeNoting::default();
    let mut keys = typed("/x");
    keys.push(PickKey::Escape);

    let picked = pick(
        menu(&targets),
        &keymap,
        &mut Inputs::typing(&keys),
        &mut Notes::default(),
    );

    assert_eq!(picked, Ok(None));
    assert_eq!(
        *keymap.modes.borrow(),
        [Mode::Browsing, Mode::Searching, Mode::Searching]
    );
}
