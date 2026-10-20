use herdr_picker_domain::{Size, popup_size};

use crate::JumpError;

pub trait PopupHost {
    fn window_size(&mut self) -> Result<Size, JumpError>;
    fn open(&mut self, size: Size) -> Result<(), JumpError>;
}

pub fn open_picker(host: &mut impl PopupHost, content_columns: usize) -> Result<(), JumpError> {
    let window = host.window_size()?;
    host.open(popup_size(window, content_columns))
}

#[cfg(test)]
mod tests;
