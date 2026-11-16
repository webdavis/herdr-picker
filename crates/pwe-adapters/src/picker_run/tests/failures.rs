use super::*;

#[test]
fn a_bad_row_ends_the_finders_input_after_its_error_is_written_for_the_footer() {
    let sandbox = Sandbox::new("bad-row");
    let seen = sandbox.0.join("seen");
    let finder = format!(
        "cat >/dev/null; cat '{}' >'{}'",
        picker_error_file(&sandbox.cache()).display(),
        seen.display()
    );

    let picked = sandbox.run(
        r#"echo '{"label":"alpha","enter":{"insert":"a"}}'; echo oops; sleep 30"#,
        &finder,
    );

    let refusal = picked.expect_err("a bad row is refused");
    assert_eq!(
        fs::read_to_string(seen).expect("the error the finder read"),
        refusal.to_string()
    );
    assert!(
        refusal
            .to_string()
            .starts_with("line 2 is not a picker row: "),
        "{refusal}"
    );
}

#[test]
fn a_picker_command_that_fails_is_refused_and_its_error_reaches_the_finder() {
    let sandbox = Sandbox::new("failed");
    let seen = sandbox.0.join("seen");
    let finder = format!(
        "cat >/dev/null; cat '{}' >'{}'",
        picker_error_file(&sandbox.cache()).display(),
        seen.display()
    );

    let picked = sandbox.run(
        r#"echo '{"label":"alpha","enter":{"insert":"a"}}'; exit 3"#,
        &finder,
    );

    let expected = "the picker command stopped with exit status: 3".to_string();
    assert_eq!(
        picked.map_err(|refusal| refusal.to_string()),
        Err(expected.clone())
    );
    assert_eq!(
        fs::read_to_string(seen).expect("the error the finder read"),
        expected
    );
}

#[test]
fn a_picker_command_that_cannot_start_is_refused_naming_it() {
    let missing = Path::new("/nonexistent/pickers/files.sh");

    let refusal = run_picker(
        &mut Command::new(missing),
        Path::new("/bin/sh"),
        &[],
        Path::new("/tmp/unused"),
    )
    .expect_err("a missing command is refused");

    assert!(
        refusal
            .to_string()
            .starts_with("cannot start /nonexistent/pickers/files.sh: "),
        "{refusal}"
    );
}

#[test]
fn what_a_failing_picker_says_on_stderr_reaches_the_finder_with_its_status() {
    let sandbox = Sandbox::new("said");

    let picked = sandbox.run(
        "echo 'This picker needs a Git repository.' >&2; exit 1",
        "cat >/dev/null",
    );

    assert_eq!(
        picked.map_err(|refusal| refusal.to_string()),
        Err(
            "the picker command stopped with exit status: 1: This picker needs a Git repository."
                .to_string()
        )
    );
}
