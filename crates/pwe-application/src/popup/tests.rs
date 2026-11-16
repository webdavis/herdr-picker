use super::*;

struct Host {
    window_size: Result<Size, JumpError>,
    opened: Vec<Size>,
}

impl Host {
    fn of(window_size: Result<Size, JumpError>) -> Self {
        Self {
            window_size,
            opened: Vec::new(),
        }
    }
}

impl PopupHost for Host {
    fn window_size(&mut self) -> Result<Size, JumpError> {
        self.window_size.clone()
    }

    fn open(&mut self, size: Size) -> Result<(), JumpError> {
        self.opened.push(size);
        Ok(())
    }
}

#[test]
fn the_picker_opens_at_the_size_its_window_and_widest_row_call_for() {
    let mut host = Host::of(Ok(Size {
        columns: 175,
        rows: 65,
    }));

    assert_eq!(open_picker(&mut host, 80), Ok(()));
    assert_eq!(
        host.opened,
        [Size {
            columns: 83,
            rows: 26
        }]
    );
}

#[test]
fn a_window_herdr_cannot_report_opens_no_popup() {
    let unreachable = JumpError::Unreachable("no socket".to_string());
    let mut host = Host::of(Err(unreachable.clone()));

    assert_eq!(open_picker(&mut host, 40), Err(unreachable));
    assert!(host.opened.is_empty());
}
