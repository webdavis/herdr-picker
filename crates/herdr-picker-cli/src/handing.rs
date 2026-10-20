use std::env;
use std::path::Path;

use herdr_picker_adapters::{DEADLINE, SocketPanes, hand_over};
use herdr_picker_protocol::{SOCKET_PATH_VARIABLE, parse_instruction};

use crate::command::{CommandError, config_path, failed};
use crate::{pick, picker, picker_popup};

// A shell key prints its result for the bash function to apply in that shell, except a result
// for a new pane, which only herdr can open.
pub(crate) fn answer(
    socket: Option<&Path>,
    instruction: Option<String>,
) -> Result<(), CommandError> {
    let Some(instruction) = instruction else {
        return Ok(());
    };
    let for_new_pane = parse_instruction(&instruction).is_some_and(|parsed| parsed.pane.is_some());
    let shell_pane = env::var("HERDR_PANE_ID")
        .ok()
        .filter(|pane| !pane.is_empty());
    match (for_new_pane, socket, shell_pane) {
        (false, _, _) => println!("{instruction}"),
        (true, Some(socket), Some(pane)) => {
            let mut panes = SocketPanes::at(socket, DEADLINE);
            hand_over(&mut panes, &pane, &instruction, &picker::editor()).map_err(failed)?;
        }
        (true, _, _) => return Err(failed("a new pane opens only inside herdr")),
    }
    Ok(())
}

// The launcher's picker, run once the launcher popup is gone, its result handed to the pane the
// launcher opened over. herdr shows a refusal as a notification, since no terminal is left.
pub(crate) fn deliver(name: &str, target: &str, spawner: &str) -> Result<(), CommandError> {
    pick::wait_for_spawner(spawner.parse().map_err(|_| CommandError::Usage)?);
    let socket = picker_popup::popup_socket(env::var(SOCKET_PATH_VARIABLE).ok())
        .ok_or_else(|| failed(format!("{SOCKET_PATH_VARIABLE} is unset")))?;
    let mut panes = SocketPanes::at(&socket, DEADLINE);
    let refused = |refusal| {
        Err(failed(format!(
            "herdr would not open the picker: {refusal}"
        )))
    };
    let outcome =
        picker_popup::in_popup(&socket, name, &config_path(), refused).and_then(|instruction| {
            match instruction {
                Some(line) => {
                    hand_over(&mut panes, target, &line, &picker::editor()).map_err(failed)
                }
                None => Ok(()),
            }
        });
    if let Err(CommandError::Failed(reason)) = &outcome {
        let _ = panes.notify(&format!("{name}: {reason}"));
    }
    outcome
}
