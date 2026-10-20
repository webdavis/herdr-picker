use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::picker_files::picker_error_file;

use herdr_picker_domain::Action;

use super::*;

const THREE_ROWS: &str = r#"{"label":"alpha","enter":{"insert":"a"}}
{"label":"beta","preview":"the second","enter":{"cd":"/opt/beta"}}
{"label":"gamma","enter":{"run":"echo gamma"}}"#;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-picker-run-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn picker(&self, body: &str) -> PathBuf {
        let path = self.0.join("picker.sh");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("a picker script");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable");
        path
    }

    fn cache(&self) -> PathBuf {
        self.0.join("rows.jsonl")
    }

    fn run(&self, picker: &str, finder: &str) -> Result<Option<PickerRow>, PickerRunError> {
        let finder_arguments = ["-c".to_string(), finder.to_string()];
        run_picker(
            &self.picker(picker),
            Path::new("/bin/sh"),
            &finder_arguments,
            &self.cache(),
        )
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_finder_reads_index_tab_label_lines_and_its_pick_is_the_row_returned() {
    let sandbox = Sandbox::new("pick");
    let seen = sandbox.0.join("seen");
    let finder = format!("tee '{}' | grep beta", seen.display());

    let picked = sandbox.run(&format!("cat <<'ROWS'\n{THREE_ROWS}\nROWS"), &finder);

    assert_eq!(
        picked,
        Ok(Some(PickerRow {
            label: "beta".to_string(),
            preview: Some("the second".to_string()),
            enter: Action::Cd("/opt/beta".to_string()),
            group: None,
        }))
    );
    assert_eq!(
        fs::read_to_string(seen).expect("the finder's input"),
        "0\talpha\n1\tbeta\n2\tgamma\n"
    );
    assert_eq!(
        fs::read_to_string(sandbox.cache()).expect("the cache"),
        format!("{THREE_ROWS}\n")
    );
}

#[test]
fn a_finder_that_picks_nothing_returns_no_row() {
    let sandbox = Sandbox::new("escape");

    let picked = sandbox.run(
        &format!("cat <<'ROWS'\n{THREE_ROWS}\nROWS"),
        "cat >/dev/null; exit 130",
    );

    assert_eq!(picked, Ok(None));
}

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
fn a_pick_made_while_the_picker_still_prints_stops_the_picker() {
    let sandbox = Sandbox::new("slow");
    let started = Instant::now();

    let picked = sandbox.run(
        r#"echo '{"label":"alpha","enter":{"insert":"a"}}'; sleep 30; echo late"#,
        "head -n 1",
    );

    assert_eq!(
        picked.map(|row| row.map(|row| row.label)),
        Ok(Some("alpha".to_string()))
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the picker was stopped"
    );
}

#[test]
fn a_picker_command_that_cannot_start_is_refused_naming_it() {
    let missing = Path::new("/nonexistent/pickers/files.sh");

    let refusal = run_picker(missing, Path::new("/bin/sh"), &[], Path::new("/tmp/unused"))
        .expect_err("a missing command is refused");

    assert!(
        refusal
            .to_string()
            .starts_with("cannot start /nonexistent/pickers/files.sh: "),
        "{refusal}"
    );
}
