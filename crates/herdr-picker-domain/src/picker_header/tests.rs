use super::*;
use crate::picker_view::View;

fn two_groups() -> Vec<String> {
    vec!["local".to_string(), "remote".to_string()]
}

fn titles(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn streamed() -> View {
    View::Streamed { group: None }
}

fn list(group: Option<&str>) -> View {
    View::List {
        group: group.map(str::to_string),
    }
}

fn chooser(group: Option<&str>) -> View {
    View::Chooser {
        group: group.map(str::to_string),
    }
}

#[test]
fn a_title_is_the_picker_name_in_words() {
    assert_eq!(title("git-branches"), "Git Branches");
    assert_eq!(title("my_files"), "My Files");
}

#[test]
fn the_header_is_the_trail_of_titles_then_the_source_when_the_rows_have_groups() {
    let branches = titles(&["Git Branches"]);
    assert_eq!(header(&branches, &streamed(), None, &[]), "Git Branches");
    assert_eq!(
        header(&branches, &streamed(), None, &two_groups()),
        "Git Branches | Source: all"
    );
    assert_eq!(
        header(&branches, &list(Some("remote")), None, &two_groups()),
        "Git Branches | Source: remote"
    );
    assert_eq!(
        header(&titles(&["My Files"]), &chooser(None), None, &two_groups()),
        "My Files › Choose source"
    );
    assert_eq!(
        header(
            &titles(&["Git Branches", "Files"]),
            &list(None),
            None,
            &two_groups()
        ),
        "Git Branches › Files | Source: all"
    );
}

#[test]
fn the_header_names_the_source_and_calls_the_rows_groups_groups_beside_it() {
    let files = titles(&["Files"]);

    assert_eq!(
        header(&files, &streamed(), Some("hidden"), &[]),
        "Files | Source: hidden"
    );
    assert_eq!(
        header(&files, &list(Some("local")), Some("hidden"), &two_groups()),
        "Files | Source: hidden | Group: local"
    );
    assert_eq!(
        header(&files, &list(None), Some("hidden"), &two_groups()),
        "Files | Source: hidden | Group: all"
    );
    assert_eq!(
        header(&files, &chooser(None), Some("hidden"), &[]),
        "Files › Choose source"
    );
}
