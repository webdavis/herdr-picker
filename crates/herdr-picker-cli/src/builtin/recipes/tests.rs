use std::fs;

use herdr_picker_domain::{Action, Key, PickerRow, Preview};

use crate::builtin::git::fixture::{labels, read};
use crate::builtin::tests::{Tree, printed};

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub(crate) const JUSTFILE: &str = "# Build it\nbuild target=\"debug\":\n    echo {{target}}\n\n[private]\nhidden:\n    true\n\ntest-unit name *rest:\n    echo {{name}}\n\nmod sub\n";

pub(crate) fn project(name: &str) -> Tree {
    let tree = Tree::new(name, &[]);
    fs::write(tree.0.join("justfile"), JUSTFILE).expect("a justfile");
    fs::write(tree.0.join("sub.just"), "lint:\n    echo lint\n").expect("a module");
    tree
}

fn key(name: &str) -> Key {
    Key::parse(name).expect("a key")
}

fn named<'a>(rows: &'a [PickerRow], label: &str) -> &'a PickerRow {
    rows.iter()
        .find(|row| row.label.starts_with(label))
        .expect("a row")
}

#[test]
fn every_public_recipe_and_module_recipe_is_listed_with_its_doc() {
    let tree = project("recipes-list");

    let (_, rows) = read(&printed("recipes", &[], None, &tree.places("home")).expect("rows"));

    assert_eq!(
        labels(&rows),
        vec![
            format!("build      {DIM}Build it{RESET}"),
            "test-unit".to_string(),
            "sub::lint".to_string(),
        ]
    );
}

#[test]
fn enter_prepares_the_recipe_and_ctrl_enter_runs_it_with_its_details_and_body_in_the_preview() {
    let tree = project("recipes-build");
    let justfile = tree.path("justfile");

    let (_, rows) = read(&printed("recipes", &[], None, &tree.places("home")).expect("rows"));

    let build = named(&rows, "build");
    let command = format!("just --justfile {justfile} build");
    assert_eq!(build.enter, Action::Insert(command.clone()));
    assert_eq!(
        build.action(&key("ctrl-enter")),
        Some(&Action::Run(command))
    );
    assert_eq!(
        build.preview,
        Some(Preview::Text(format!(
            "Runs        just build\nParameters  target=debug\nDefined in  {justfile}\n\n# Build it\nbuild target=\"debug\":\n    echo {{{{ target }}}}\n"
        )))
    );
}

#[test]
fn a_recipe_with_required_parameters_is_only_prepared_for_them_to_be_typed() {
    let tree = project("recipes-params");
    let justfile = tree.path("justfile");

    let (_, rows) = read(&printed("recipes", &[], None, &tree.places("home")).expect("rows"));

    let test = named(&rows, "test-unit");
    assert_eq!(
        test.enter,
        Action::Insert(format!("just --justfile {justfile} test-unit "))
    );
    assert_eq!(test.action(&key("ctrl-enter")), None);
    let Some(Preview::Text(preview)) = &test.preview else {
        panic!("a text preview");
    };
    assert!(
        preview.starts_with("Runs        just test-unit\nParameters  name (required), rest\n"),
        "{preview}"
    );
}

#[test]
fn without_a_justfile_the_picker_says_what_just_said() {
    let tree = Tree::new("recipes-none", &[]);

    let refusal = printed("recipes", &[], None, &tree.places("home"))
        .map_err(|failure| format!("{failure:?}"));

    assert_eq!(
        refusal,
        Err("Failed(\"error: no justfile found\")".to_string())
    );
}
