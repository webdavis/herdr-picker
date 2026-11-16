use std::env;
use std::io;
use std::path::PathBuf;

use pwe_adapters::{
    MacNotifier, RexNoticeScreen, RexPanes, RexWhereabouts, inbox_file, read_picker_config,
    rex_program,
};
use pwe_daemon::NoticeParts;

use crate::command::{CommandError, config_path, failed, state_dir};

const REX_ON_PATH: &str = "rex";

pub(crate) fn notice_parts() -> Result<NoticeParts, CommandError> {
    let config = read_picker_config(&config_path()).map_err(failed)?;
    let path_variable = env::var_os("PATH");
    let rex = rex_program(
        path_variable.as_deref(),
        &PathBuf::from("/Applications"),
        &config.rex.app_name,
    )
    .unwrap_or_else(|_| PathBuf::from(REX_ON_PATH));
    let pwe = env::current_exe().map_err(failed)?;
    Ok(NoticeParts {
        settings: config.notices.board,
        screen: Box::new(RexNoticeScreen::at(
            rex.clone(),
            pwe.to_string_lossy().into_owned(),
            config.notices.corner,
        )),
        notifier: Box::new(MacNotifier::new(
            config.notices.macos,
            path_variable.clone(),
        )),
        whereabouts: Box::new(RexWhereabouts::new(&config.rex.app_name, path_variable)),
        places: Box::new(RexPanes::at(rex, None)),
        inbox: inbox_file(&state_dir()),
        inbox_limits: config.inbox,
        log: Box::new(io::stderr()),
    })
}
