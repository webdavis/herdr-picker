use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;

fn entry(path: &str, line: u32, text: &str) -> QuickfixEntry {
    QuickfixEntry {
        path: path.to_string(),
        line,
        text: text.to_string(),
    }
}

#[test]
fn the_list_is_saved_privately_as_neovim_reads_it_and_read_back() {
    let directory = std::env::temp_dir().join(format!("hwj-quickfix-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    let file = directory.join("state/quickfix.json");
    let entries = vec![
        entry("/p/a.rs", 7, "error: it's broken"),
        entry("/p/b.rs", 1, "b"),
    ];

    save_quickfix(&file, &entries).expect("saved");

    assert_eq!(
        fs::read_to_string(&file).expect("the list"),
        r#"[{"filename":"/p/a.rs","lnum":7,"text":"error: it's broken"},{"filename":"/p/b.rs","lnum":1,"text":"b"}]"#
    );
    let mode = |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
    assert_eq!(
        (mode(&file), mode(&directory.join("state"))),
        (0o600, 0o700)
    );
    assert_eq!(read_quickfix(&file).expect("read back"), entries);
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn neovim_opens_with_the_saved_list_loaded_and_shown_or_at_one_of_its_entries() {
    let file = Path::new("/state/it's/quickfix.json");

    assert_eq!(
        open_quickfix(file, None),
        r#"nvim -c 'call setqflist(json_decode(join(readfile('\''/state/it'\'''\''s/quickfix.json'\''))))' -c copen"#
    );
    assert_eq!(
        open_quickfix(Path::new("/state/quickfix.json"), Some(3)),
        r#"nvim -c 'call setqflist(json_decode(join(readfile('\''/state/quickfix.json'\''))))' -c copen -c 'cc 3'"#
    );
}
