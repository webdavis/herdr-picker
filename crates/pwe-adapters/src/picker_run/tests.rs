use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::picker_files::picker_error_file;

use pwe_domain::finder::{Ending, Found};
use pwe_domain::{Action, Key};

use super::*;

mod failures;

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

    fn run(&self, picker: &str, finder: &str) -> Result<Option<Found<PickerRow>>, PickerRunError> {
        let finder_arguments = ["-c".to_string(), finder.to_string()];
        run_picker(
            &mut Command::new(self.picker(picker)),
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
fn the_finder_reads_index_tab_label_lines_and_answers_its_query_key_and_marked_rows() {
    let sandbox = Sandbox::new("pick");
    let seen = sandbox.0.join("seen");
    let finder = format!(
        "tee '{}' | grep -v beta | {{ printf 'ga\\nctrl-o\\n'; cat; }}",
        seen.display()
    );

    let picked = sandbox.run(&format!("cat <<'ROWS'\n{THREE_ROWS}\nROWS"), &finder);

    let row = |label: &str, enter: Action| PickerRow {
        label: label.to_string(),
        preview: None,
        enter,
        keys: Vec::new(),
        group: None,
    };
    assert_eq!(
        picked,
        Ok(Some(Found {
            query: "ga".to_string(),
            ending: Ending::Key(Key::parse("ctrl-o").expect("a row key")),
            rows: vec![
                row("alpha", Action::Insert("a".to_string())),
                row("gamma", Action::Run("echo gamma".to_string())),
            ],
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
fn escape_ends_the_step_by_going_back() {
    let sandbox = Sandbox::new("back");

    let picked = sandbox.run(
        &format!("cat <<'ROWS'\n{THREE_ROWS}\nROWS"),
        "cat >/dev/null; printf 'al\\nesc\\n'",
    );

    assert_eq!(
        picked.map(|picked| picked.map(|picked| (picked.query, picked.ending))),
        Ok(Some(("al".to_string(), Ending::Back)))
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
fn a_pick_made_while_the_picker_still_prints_stops_the_picker() {
    let sandbox = Sandbox::new("slow");
    let started = Instant::now();

    let picked = sandbox.run(
        r#"echo '{"label":"alpha","enter":{"insert":"a"}}'; sleep 30; echo late"#,
        "printf '\\nenter\\n'; head -n 1",
    );

    assert_eq!(
        picked.map(|picked| picked.map(|picked| picked.rows[0].label.clone())),
        Ok(Some("alpha".to_string()))
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the picker was stopped"
    );
}

#[test]
fn a_header_line_from_the_picker_becomes_the_step_header_and_is_no_row() {
    let sandbox = Sandbox::new("header");
    let picker =
        r#"printf '%s\n' '{"header":"Pull into main?"}' '{"label":"Pull","enter":"none"}'"#;

    let picked = sandbox.run(picker, "cat > /dev/null; printf '\\nenter\\n0\\tPull\\n'");

    assert_eq!(
        picked.expect("a pick").map(|picked| picked.rows.len()),
        Some(1)
    );
    assert_eq!(
        crate::picker_files::read_step_header(&sandbox.cache()),
        Some("Pull into main?".to_string())
    );
    assert_eq!(
        fs::read_to_string(sandbox.cache()).expect("the cache"),
        "{\"label\":\"Pull\",\"enter\":\"none\"}\n"
    );
}

#[test]
fn the_rows_picked_are_the_ones_the_cache_holds_when_the_finder_closes() {
    let sandbox = Sandbox::new("relisted");
    let cache = sandbox.cache();
    let finder = format!(
        "cat > /dev/null; printf '%s\\n' '{{\"label\":\"late\",\"enter\":\"none\"}}' > '{}'; printf 'la\\nenter\\n0\\tlate\\n'",
        cache.display()
    );

    let picked = sandbox
        .run(
            &format!("printf '%s\\n' '{}'", r#"{"label":"early","enter":"none"}"#),
            &finder,
        )
        .expect("a pick")
        .expect("a row");

    assert_eq!(picked.rows[0].label, "late");
}

#[test]
fn every_file_a_run_leaves_only_its_owner_reads() {
    let sandbox = Sandbox::new("private");
    let picker = r#"printf '%s\n' '{"header":"Pick"}' '{"label":"a","enter":"none"}' 'not a row'"#;

    let _ = sandbox.run(picker, "cat > /dev/null; printf '\\nenter\\n'");

    let cache = sandbox.cache();
    let modes: Vec<u32> = [
        cache.clone(),
        crate::picker_files::picker_said_file(&cache),
        crate::picker_files::picker_header_file(&cache),
        picker_error_file(&cache),
    ]
    .iter()
    .map(|file| fs::metadata(file).expect("a run file").permissions().mode() & 0o777)
    .collect();
    assert_eq!(modes, vec![0o600; 4]);
}
