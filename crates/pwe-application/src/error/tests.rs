use super::*;

#[test]
fn every_error_renders_on_one_line() {
    for error in [
        JumpError::Unreachable("no such file".to_string()),
        JumpError::Transport("timed out".to_string()),
        JumpError::Malformed("not JSON".to_string()),
        JumpError::Server {
            code: "not_found".to_string(),
            message: "gone".to_string(),
        },
        JumpError::BothPathsFailed {
            cli: "herdr unreachable: no binary".to_string(),
            socket: "herdr unreachable: no socket".to_string(),
        },
    ] {
        let rendered = error.to_string();
        assert!(!rendered.contains('\n'), "multi-line: {rendered}");
        assert!(!rendered.is_empty());
    }
}

#[test]
fn no_error_names_the_pane_host_it_came_from() {
    for error in [
        JumpError::Unreachable("rex is not on PATH".to_string()),
        JumpError::Transport("rex send exited 1".to_string()),
        JumpError::Malformed("not JSON".to_string()),
        JumpError::Server {
            code: "not_found".to_string(),
            message: "gone".to_string(),
        },
    ] {
        assert!(!error.to_string().contains("herdr"), "{error}");
    }
}
