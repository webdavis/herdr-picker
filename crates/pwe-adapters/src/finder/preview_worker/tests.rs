use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::mpsc;
use std::time::Duration;

use pwe_domain::Preview;
use pwe_domain::finder::Input;

use super::PreviewWorker;

const PATIENCE: Duration = Duration::from_secs(10);

#[test]
fn a_preview_answers_for_the_row_it_was_asked_for() {
    let (answers, answered) = mpsc::channel();
    let worker = PreviewWorker::start(answers, None);

    worker.ask(3, Preview::Text("hello".to_string()), 40);

    assert_eq!(
        answered.recv_timeout(PATIENCE),
        Ok(Input::PreviewArrived {
            row: 3,
            text: "hello".to_string()
        })
    );
}

#[test]
fn a_file_preview_gets_the_width_it_was_asked_for() {
    let directory = std::env::temp_dir().join(format!("pwe-preview-worker-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("a sandbox");
    let bat = directory.join("bat");
    fs::write(&bat, "#!/bin/sh\necho \"$@\"\n").expect("a fake bat");
    fs::set_permissions(&bat, fs::Permissions::from_mode(0o700)).expect("executable");
    let file = directory.join("note.txt");
    fs::write(&file, "words\n").expect("a file");
    let (answers, answered) = mpsc::channel();
    let worker = PreviewWorker::start(answers, Some(OsString::from(&directory)));

    worker.ask(0, Preview::Path(file.to_string_lossy().into_owned()), 33);

    let answer = answered.recv_timeout(PATIENCE);
    let _ = fs::remove_dir_all(&directory);
    assert!(
        matches!(&answer, Ok(Input::PreviewArrived { text, .. }) if text.contains("--terminal-width=33")),
        "{answer:?}"
    );
}
