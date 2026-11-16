use super::*;

#[test]
fn a_config_without_pickers_runs_the_builtin_ones_on_their_keys() {
    let sandbox = TempDirectory::new("no-pickers");
    let path = sandbox.write(FIVE_WORKSPACES_DECLARED_OUT_OF_ORDER);

    assert_eq!(
        read_config(&path).expect("a readable config").pickers,
        vec![
            builtin(
                "files",
                "ctrl-t f",
                "Files here, hidden ones, at home, on the whole disk, or used lately"
            ),
            builtin("folders", "ctrl-t d", "Folders to change into"),
            builtin(
                "bookmark",
                "ctrl-t ctrl-b",
                "Bookmark this folder for the folders picker"
            ),
            builtin(
                "status",
                "ctrl-g s",
                "Every file Git has an opinion about, and the stashes"
            ),
            builtin(
                "history",
                "ctrl-g l",
                "Commits, the reflog, or the commits whose changes match what is typed"
            ),
            builtin(
                "branches",
                "ctrl-g b",
                "Branches, tags and worktrees, or the repos with something to prune"
            ),
            builtin("contents", "ctrl-g /", "Lines matching what is typed"),
            builtin(
                "git-config",
                "ctrl-g v",
                "Git settings, where each comes from, and which value wins"
            ),
            builtin("recipes", "ctrl-t j", "Recipes in this project's justfile"),
            builtin(
                "tests",
                "ctrl-t t",
                "Tests in this project, compiler and linter errors, or the saved quickfix list"
            ),
            builtin(
                "processes",
                "ctrl-t p",
                "Processes and the sockets they hold, services, or the last hour of logs"
            ),
            builtin(
                "network",
                "ctrl-t n",
                "Devices on the network, saved captures, checks of a URL, or network commands"
            ),
            builtin(
                "data",
                "ctrl-t i",
                "JSON, YAML, TOON and SQLite files here, or the clipboard, by path and value"
            ),
            builtin(
                "agents",
                "ctrl-t a",
                "Agents and tabs in herdr, the files that instruct agents here, saved failures, and past sessions"
            ),
            builtin(
                "pickers",
                "ctrl-t ;",
                "Every picker by its key, or every key binding in the shell"
            ),
            builtin(
                "apple",
                "ctrl-t m",
                "Messages through BlueBubbles, Contacts, and Shortcuts"
            ),
            builtin("tabs", "ctrl-t w", "Open Arc tabs, by Space"),
        ]
    );
}

#[test]
fn without_a_config_file_the_pickers_are_the_builtin_ones() {
    let sandbox = TempDirectory::new("picker-config-missing");

    let config = read_picker_config(&sandbox.0.join("config.toml")).expect("the defaults");

    assert_eq!(
        names(&config.pickers),
        [
            "files",
            "folders",
            "bookmark",
            "status",
            "history",
            "branches",
            "contents",
            "git-config",
            "recipes",
            "tests",
            "processes",
            "network",
            "data",
            "agents",
            "pickers",
            "apple",
            "tabs"
        ]
    );
    assert_eq!(config.targets, Vec::new());
}
