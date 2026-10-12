use super::*;

#[test]
fn a_screen_that_cannot_show_stops_the_popup_before_any_key_is_read() {
    let targets = workspaces();
    let mut keys = Keys::typing(&typed("d"));
    let mut screen = Notes {
        failure: Some("terminal gone"),
        ..Notes::default()
    };

    let picked = pick(menu(&targets), &ReadlineKeymap, &mut keys, &mut screen);

    assert_eq!(picked, Err(PickError::Show("terminal gone".to_string())));
    assert_eq!(keys.0.len(), 1, "a key was read after the show failed");
}

#[test]
fn a_key_that_cannot_be_read_is_reported() {
    let targets = workspaces();

    let picked = pick(
        menu(&targets),
        &ReadlineKeymap,
        &mut Keys::typing(&[]),
        &mut Notes::default(),
    );

    assert_eq!(picked, Err(PickError::ReadKey("no more keys".to_string())));
}

#[test]
fn every_error_renders_on_one_line_naming_what_failed() {
    assert_eq!(
        PickError::Show("broken pipe".to_string()).to_string(),
        "cannot show the menu: broken pipe"
    );
    assert_eq!(
        PickError::ReadKey("no tty".to_string()).to_string(),
        "cannot read a key: no tty"
    );
}
