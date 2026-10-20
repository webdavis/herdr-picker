use herdr_picker_domain::View;

use super::*;

fn state(view: View) -> ViewState {
    ViewState {
        view,
        titles: vec!["Files".to_string()],
        description: "Every file".to_string(),
        source: Some("hidden".to_string()),
        sources: vec!["this folder".to_string(), "hidden".to_string()],
    }
}

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-picker-files-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn cache(&self) -> PathBuf {
        self.0.join("picker-rows-7.jsonl")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_runs_view_and_error_sit_beside_its_rows_cache() {
    let cache = Path::new("/state/picker-rows-7.jsonl");

    assert_eq!(
        picker_view_file(cache),
        PathBuf::from("/state/picker-rows-7.view")
    );
    assert_eq!(
        picker_error_file(cache),
        PathBuf::from("/state/picker-rows-7.error")
    );
}

#[test]
fn a_run_without_a_view_file_shows_the_streamed_rows_until_one_is_written() {
    let sandbox = Sandbox::new("view");
    let cache = sandbox.cache();
    assert_eq!(read_view(&cache).view, View::Streamed { group: None });

    let chooser = state(View::Chooser {
        group: Some("local".to_string()),
    });
    write_view(&cache, &chooser).expect("the view is written");

    assert_eq!(read_view(&cache), chooser);
}

#[test]
fn removing_a_runs_files_leaves_none_behind() {
    let sandbox = Sandbox::new("remove");
    let cache = sandbox.cache();
    fs::write(&cache, "{}\n").expect("a cache");
    write_view(&cache, &state(View::Streamed { group: None })).expect("a view");
    fs::write(picker_error_file(&cache), "line 2 is bad").expect("an error");
    fs::write(picker_header_file(&cache), "main | 1 staged").expect("a header");
    fs::write(picker_command_file(&cache), "{}").expect("a command");
    assert_eq!(read_run_error(&cache), Some("line 2 is bad".to_string()));

    remove_run_files(&cache);

    assert_eq!(
        fs::read_dir(&sandbox.0).expect("the sandbox").count(),
        0,
        "every run file is gone"
    );
}

#[test]
fn the_first_view_of_a_run_creates_the_state_directory() {
    let sandbox = Sandbox::new("fresh");
    let cache = sandbox.0.join("state/picker-rows-7.jsonl");

    write_view(&cache, &state(View::Streamed { group: None })).expect("the view is written");

    assert_eq!(read_view(&cache), state(View::Streamed { group: None }));
}

#[test]
fn a_step_header_is_read_from_beside_the_rows_cache_and_absent_until_written() {
    let sandbox = Sandbox::new("header");
    let cache = sandbox.cache();

    assert_eq!(read_step_header(&cache), None);
    fs::write(
        picker_header_file(&cache),
        "main → origin/main ↑1 | 2 staged",
    )
    .expect("a header");

    assert_eq!(
        read_step_header(&cache),
        Some("main → origin/main ↑1 | 2 staged".to_string())
    );
}
