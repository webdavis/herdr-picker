use std::path::Path;
use std::process::Command;

const VARIABLES_LOCAL_TO_A_REPOSITORY: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

pub(crate) fn git_in_folder(folder: &Path) -> Command {
    in_folder("git", folder)
}

pub(super) fn wt_in_folder(folder: &Path) -> Command {
    in_folder("wt", folder)
}

fn in_folder(program: &str, folder: &Path) -> Command {
    let mut command = Command::new(program);
    command.arg("-C").arg(folder);
    for variable in VARIABLES_LOCAL_TO_A_REPOSITORY {
        command.env_remove(variable);
    }
    command
}

#[cfg(test)]
mod tests;
