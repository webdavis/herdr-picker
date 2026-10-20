use std::fs;

use super::*;

fn sandbox(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("hwj-bookmarks-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    directory
}

#[test]
fn a_bookmark_is_saved_once_and_read_back_in_the_order_added() {
    let directory = sandbox("add");
    let file = directory.join("state/bookmarks");

    assert_eq!(
        add_bookmark(&file, Path::new("/work/repo")).ok(),
        Some(true)
    );
    assert_eq!(
        add_bookmark(&file, Path::new("/opt/notes")).ok(),
        Some(true)
    );
    assert_eq!(
        add_bookmark(&file, Path::new("/work/repo")).ok(),
        Some(false)
    );

    assert_eq!(
        read_bookmarks(&file),
        [PathBuf::from("/work/repo"), PathBuf::from("/opt/notes")]
    );
    assert_eq!(
        fs::read_to_string(&file).expect("the bookmarks file"),
        "/work/repo\n/opt/notes\n"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn no_bookmarks_file_reads_as_no_bookmarks() {
    assert_eq!(
        read_bookmarks(&sandbox("missing").join("bookmarks")),
        Vec::<PathBuf>::new()
    );
}
