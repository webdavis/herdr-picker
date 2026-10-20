use std::process::Command;

use herdr_picker_domain::{Action, Key, Preview};

use crate::builtin::data::tests::{DIM, RESET, file_rows};
use crate::builtin::git::fixture::labels;
use crate::builtin::tests::Tree;

#[test]
fn a_database_lists_its_tables_views_and_indexes_as_read_only_queries() {
    let tree = Tree::new("data-sqlite", &[]);
    let database = tree.path("app.db");
    let made = Command::new("sqlite3")
        .arg(&database)
        .arg(concat!(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
            "CREATE INDEX users_name ON users (name);",
            "CREATE VIEW names AS SELECT name FROM users;"
        ))
        .status()
        .expect("sqlite3 runs");
    assert!(made.success());

    let (_, rows) = file_rows(&tree, "app.db", "paths", None);

    assert_eq!(
        labels(&rows),
        vec![
            "users_name".to_string(),
            format!("users  {DIM}2 columns{RESET}"),
            format!("names  {DIM}1 column{RESET}"),
        ]
    );
    let groups: Vec<_> = rows
        .iter()
        .map(|row| row.group.clone().unwrap_or_default())
        .collect();
    assert_eq!(groups, ["index", "table", "view"]);
    let query = format!(
        "sqlite3 -readonly -header -column {database} 'SELECT * FROM \"users\" LIMIT 100;'"
    );
    assert_eq!(rows[1].enter, Action::Insert(query.clone()));
    assert_eq!(
        rows[1].action(&Key::parse("ctrl-enter").expect("a key")),
        Some(&Action::Run(query.clone()))
    );
    assert_eq!(
        rows[1].preview,
        Some(Preview::Text(format!(
            "{query}\n\nCREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)\n\n\
             Columns:\nid  INTEGER  PRIMARY KEY\nname  TEXT  NOT NULL\n\n\
             Indexes: users_name\nEnter prepares this command."
        )))
    );
}
