use super::*;

#[test]
fn what_osascript_says_on_failure_is_the_scripts_own_reason() {
    assert_eq!(
        reason("execution error: Error: Error: arc is not running (-2700)\n"),
        "arc is not running"
    );
    assert_eq!(
        reason("execution error: Error: Error: Application can't be found. (-2700)"),
        "Application can't be found."
    );
    assert_eq!(reason("osascript: no such file"), "osascript: no such file");
}
