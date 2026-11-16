use super::*;

#[test]
fn yaml_is_read_through_yq_into_the_same_rows() {
    let tree = holding(
        "data-yaml",
        "ci.yml",
        "jobs:\n  build:\n    runs: [linux]\n",
    );

    let (_, rows) = file_rows(&tree, "ci.yml", "paths", None);

    assert_eq!(
        labels(&rows)[3..],
        [
            format!(".jobs.build.runs  {DIM}[1 item]{RESET}"),
            format!(".jobs.build.runs[0]  {DIM}\"linux\"{RESET}"),
        ]
    );
    assert_eq!(
        on(&rows[4], "ctrl-enter"),
        Action::Edit(vec![Location {
            path: tree.path("ci.yml"),
            line: Some(3)
        }])
    );
}

#[test]
fn toon_is_read_in_process_into_the_same_rows() {
    let tree = holding(
        "data-toon",
        "users.toon",
        "users[2]{name,id}:\n  ann,1\n  bob,2\n",
    );

    let (_, rows) = file_rows(&tree, "users.toon", "paths", None);

    assert_eq!(
        labels(&rows),
        vec![
            format!(".  {DIM}{{1 key}}{RESET}"),
            format!(".users  {DIM}[2 items]{RESET}"),
            format!(".users[0]  {DIM}{{2 keys}}{RESET}"),
            format!(".users[0].name  {DIM}\"ann\"{RESET}"),
            format!(".users[0].id  {DIM}1{RESET}"),
            format!(".users[1]  {DIM}{{2 keys}}{RESET}"),
            format!(".users[1].name  {DIM}\"bob\"{RESET}"),
            format!(".users[1].id  {DIM}2{RESET}"),
        ]
    );
}

#[test]
fn a_file_that_does_not_parse_says_so_in_a_row() {
    let tree = holding("data-broken", "a.json", "{\"open\": ");

    let (_, rows) = file_rows(&tree, "a.json", "paths", None);

    assert_eq!(
        labels(&rows),
        vec![format!("{DIM}Could not read this document{RESET}")]
    );
}
