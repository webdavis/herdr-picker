use std::fs;
use std::path::PathBuf;

use super::*;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-path-preview-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// An empty PATH finds neither bat nor eza, so the preview falls back to reading the path itself.
const NO_TOOLS: Option<&OsStr> = None;

#[test]
fn a_folder_lists_its_folders_first_then_its_files() {
    let sandbox = Sandbox::new("folder");
    fs::create_dir(sandbox.0.join("src")).expect("a folder");
    fs::write(sandbox.0.join("b.txt"), "b").expect("a file");
    fs::write(sandbox.0.join("a.txt"), "a").expect("a file");

    assert_eq!(path_preview(&sandbox.0, NO_TOOLS), "src/\na.txt\nb.txt\n");
}

#[test]
fn a_text_file_shows_its_first_lines() {
    let sandbox = Sandbox::new("text");
    let file = sandbox.0.join("notes.txt");
    let lines: String = (1..=300).map(|number| format!("line {number}\n")).collect();
    fs::write(&file, lines).expect("a file");

    let preview = path_preview(&file, NO_TOOLS);

    assert_eq!(preview.lines().count(), 200);
    assert!(preview.starts_with("line 1\nline 2\n"), "{preview}");
}

#[test]
fn a_binary_file_gets_a_short_note_instead_of_its_bytes() {
    let sandbox = Sandbox::new("binary");
    let file = sandbox.0.join("image.png");
    fs::write(&file, [0x89, b'P', b'N', b'G', 0, 1, 2]).expect("a file");

    assert_eq!(path_preview(&file, NO_TOOLS), "Binary file, 7 bytes");
}

#[test]
fn a_path_that_is_gone_says_so() {
    assert_eq!(
        path_preview(Path::new("/nonexistent/notes.txt"), NO_TOOLS),
        "Cannot read /nonexistent/notes.txt: No such file or directory (os error 2)"
    );
}
