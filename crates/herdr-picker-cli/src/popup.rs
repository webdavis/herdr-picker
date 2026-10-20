use std::path::Path;

use herdr_picker_adapters::{DEADLINE, Layout, SocketPopupHost};
use herdr_picker_application::open_picker;
use herdr_picker_domain::{JumpTarget, ReadlineKeymap};

use crate::command::{CommandError, failed};
use herdr_picker_protocol::{PopupPane, SOCKET_PATH_VARIABLE};

pub(crate) fn run(
    socket_path: Option<String>,
    plugin_id: Option<String>,
    current_workspace: Option<String>,
    targets: &[JumpTarget],
) -> Result<(), CommandError> {
    let socket_path = present(socket_path, SOCKET_PATH_VARIABLE)?;
    let plugin_id = present(plugin_id, "HERDR_PLUGIN_ID")?;
    let mut host = SocketPopupHost::at(
        Path::new(&socket_path),
        DEADLINE,
        &plugin_id,
        PopupPane::workspace_picker(current_workspace.as_deref()),
    );
    open_picker(
        &mut host,
        Layout::new(ReadlineKeymap::LEGEND, targets).columns_to_fit(),
    )
    .map_err(|failure| failed(format!("could not open the picker: {failure}")))
}

fn present(value: Option<String>, variable: &str) -> Result<String, CommandError> {
    value.filter(|value| !value.is_empty()).ok_or_else(|| {
        failed(format!(
            "{variable} is unset, so herdr did not run this as the plugin's pick action"
        ))
    })
}

#[cfg(test)]
mod tests;
