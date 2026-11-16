use super::*;

const PS_LISTING: &str = "  PID TTY           TIME CMD\n101 ttys001    0:05.12 claude --resume 22222222 SHELL=/bin/bash CLAUDE_CONFIG_DIR=/home/someone/.claude-work REX_BLOCK=block:bbbbbbbb\n";

#[test]
fn the_variable_is_read_from_a_ps_listing() {
    assert_eq!(
        variable_in_ps_listing(PS_LISTING, "CLAUDE_CONFIG_DIR").as_deref(),
        Some("/home/someone/.claude-work")
    );
}

#[test]
fn an_unset_variable_reads_as_absent() {
    assert_eq!(variable_in_ps_listing(PS_LISTING, "CODEX_HOME"), None);
    assert_eq!(variable_in_ps_listing("", "CLAUDE_CONFIG_DIR"), None);
}

#[test]
fn the_variable_is_read_from_a_linux_environ_file() {
    let environ = b"SHELL=/bin/bash\0CLAUDE_CONFIG_DIR=/home/someone/.claude-work\0LANG=C\0";

    assert_eq!(
        variable_in_environ_file(environ, "CLAUDE_CONFIG_DIR").as_deref(),
        Some("/home/someone/.claude-work")
    );
    assert_eq!(variable_in_environ_file(environ, "CLAUDE"), None);
}
