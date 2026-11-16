use super::*;

const LISTED: &str = r#"{"schema":2,"items":[
{"branch":"main","head":{"sha":"aaa111","short_sha":"aaa","subject":"init"},
 "worktree":{"path":"/r","main":true,"current":true,"detached":false},
 "display":{"statusline":"main  \u001b[2m^\u001b[22m"}},
{"branch":"merged","head":{"sha":"aaa111","short_sha":"aaa","subject":"init"},
 "worktree":{"path":"/w/merged","main":false,"current":false,"detached":false},
 "display":{"statusline":"merged  \u001b[2m_\u001b[22m"}},
{"branch":"feat","head":{"sha":"bbb222","short_sha":"bbb","subject":"feat"},
 "display":{"statusline":"feat  \u001b[2m↑\u001b[22m"}}
]}"#;

#[test]
fn worktrunk_lists_each_branch_with_its_status_and_the_worktree_holding_it() {
    let items = parse(LISTED).expect("items");

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].branch.as_deref(), Some("main"));
    assert_eq!(items[0].subject, "init");
    assert_eq!(items[0].status, "\u{1b}[2m^\u{1b}[22m");
    assert_eq!(
        items[1].worktree,
        Some(Worktree {
            path: "/w/merged".to_string(),
            main: false,
            current: false
        })
    );
    assert_eq!(items[2].worktree, None);
    assert_eq!(items[2].status, "\u{1b}[2m↑\u{1b}[22m");
}

#[test]
fn output_that_is_not_worktrunks_list_is_refused() {
    assert!(parse("[]").is_none());
    assert!(parse("not json").is_none());
}
