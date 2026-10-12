use super::*;

#[test]
fn a_workspaces_key_picks_it_after_one_frame() {
    let targets = workspaces();
    let mut screen = Notes::default();

    let picked = pick(menu(&targets), &mut Keys::typing(&typed("d")), &mut screen);

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
        &mut Keys::typing(&[PickKey::Next, PickKey::Next, PickKey::Enter]),
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
fn escape_closes_with_nothing_picked() {
    let targets = workspaces();
    let mut screen = Notes::default();

    let picked = pick(
        menu(&targets),
        &mut Keys::typing(&[PickKey::Escape]),
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
        &mut Keys::typing(&typed("z")),
        &mut Notes::default(),
    );

    assert_eq!(picked, Ok(None));
}
