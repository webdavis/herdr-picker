use std::fs;

use super::*;

#[test]
fn a_check_runs_in_its_folder_and_what_it_printed_and_how_it_ended_are_saved() {
    let directory = std::env::temp_dir()
        .canonicalize()
        .expect("the temp directory")
        .join(format!("hwj-capture-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("a folder");
    let output = directory.join("captures/diagnostics-rust.json");
    let argv = ["sh", "-c", "pwd; echo oops >&2; exit 3"].map(str::to_string);

    let saved = run_capture(&output, &directory, "diagnostics", "rust", &argv).expect("saved");

    let expected = Capture {
        category: "diagnostics".to_string(),
        cwd: directory.to_string_lossy().into_owned(),
        framework: "rust".to_string(),
        argv: argv.to_vec(),
        exit_code: 3,
        stdout: format!("{}\n", directory.display()),
        stderr: "oops\n".to_string(),
    };
    assert_eq!(saved, expected);
    assert_eq!(
        read_captures(&output.with_file_name(""), "diagnostics"),
        vec![expected]
    );
    assert_eq!(
        read_captures(&directory.join("captures"), "tests"),
        Vec::new()
    );
    let _ = fs::remove_dir_all(&directory);
}
