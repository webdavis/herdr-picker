use crate::finder::fixtures::{effects_of, finder_of, finder_with, found_by, press, row, typed};
use crate::finder::{Effect, Ending, Finder, Input, Remark, step};
use crate::{
    Action, Fuzzy, Key, Location, PickerRow, QuickfixEntry, View, ViewState, quickfix_entries,
};

fn claiming(label: &str, key: &str, action: Action) -> PickerRow {
    let key = Key::parse(key).expect("a bindable key");
    PickerRow {
        keys: vec![(key, action)],
        ..row(label)
    }
}

fn in_group(label: &str, group: &str) -> PickerRow {
    PickerRow {
        group: Some(group.to_string()),
        ..row(label)
    }
}

fn editing(label: &str, path: &str) -> PickerRow {
    let location = Location {
        path: path.to_string(),
        line: Some(3),
    };
    PickerRow {
        enter: Action::Edit(vec![location]),
        ..row(label)
    }
}

fn with_sources(rows: Vec<PickerRow>, sources: &[&str]) -> Finder<Fuzzy> {
    let state = ViewState {
        view: View::Streamed { group: None },
        titles: vec!["Files".to_string()],
        description: String::new(),
        source: None,
        sources: sources.iter().map(|source| source.to_string()).collect(),
    };
    let finder = Finder::new(Fuzzy).with_view(state);
    let (finder, _) = step(finder, Input::RowsArrived(rows));
    step(finder, Input::RowsEnded).0
}

#[test]
fn a_key_no_row_claims_moves_the_cursor_in_the_query_as_fzf_does() {
    let finder = typed(finder_with(&["xabc"]), "abc");
    let finder = typed(press(finder, &["ctrl-a"]), "x");

    assert_eq!(finder.query(), "xabc");
}

#[test]
fn a_key_a_row_claims_wins_over_its_readline_meaning() {
    let finder = finder_of(vec![claiming("one", "ctrl-a", Action::Run("ls".into()))]);

    let found = found_by(finder, "ctrl-a");

    assert_eq!(
        found.ending,
        Ending::Key(Key::parse("ctrl-a").expect("a key"))
    );
    assert_eq!(found.rows, [0]);
}

#[test]
fn readline_keys_edit_the_query_as_fzf_does() {
    let edited = |keys: &[&str]| press(typed(finder_with(&["x"]), "one two"), keys).query();

    assert_eq!(edited(&["ctrl-w"]), "one ");
    assert_eq!(edited(&["ctrl-u"]), "");
    assert_eq!(edited(&["ctrl-w", "ctrl-y", "ctrl-y"]), "one twotwo");
    assert_eq!(edited(&["alt-b", "ctrl-k", "ctrl-d"]), "one wo");
    assert_eq!(edited(&["bspace", "left", "del"]), "one t");
    assert_eq!(edited(&["ctrl-a", "alt-f", "alt-d"]), "one");
}

#[test]
fn ctrl_j_and_ctrl_k_move_the_cursor_when_no_row_claims_them() {
    let finder = press(
        finder_with(&["one", "two", "three"]),
        &["ctrl-j", "ctrl-j", "ctrl-k"],
    );

    assert_eq!(found_by(finder, "enter").rows, [1]);
}

#[test]
fn escape_ends_the_step_by_going_back() {
    assert_eq!(found_by(finder_with(&["one"]), "esc").ending, Ending::Back);
}

#[test]
fn ctrl_c_ctrl_g_and_ctrl_d_on_an_empty_query_close_without_finding() {
    for key in ["ctrl-c", "ctrl-g", "ctrl-d"] {
        assert_eq!(
            effects_of(finder_with(&["one"]), key),
            [Effect::CloseWithoutFinding]
        );
    }
}

#[test]
fn a_row_may_claim_ctrl_g() {
    let finder = finder_of(vec![claiming("one", "ctrl-g", Action::Reload)]);

    assert!(matches!(found_by(finder, "ctrl-g").ending, Ending::Key(_)));
}

#[test]
fn enter_on_a_row_with_nothing_to_do_keeps_the_finder_open() {
    let finder = finder_of(vec![PickerRow {
        enter: Action::Nothing,
        ..row("one")
    }]);

    assert_eq!(effects_of(finder, "enter"), [Effect::Draw]);
}

#[test]
fn ctrl_space_opens_the_chooser_with_the_sources_and_groups() {
    let rows = vec![in_group("one", "docs"), in_group("two", "src")];
    let finder = press(with_sources(rows, &["home", "disk"]), &["ctrl-space"]);

    assert_eq!(finder.listed(), ["home", "disk", "all", "docs", "src"]);
}

#[test]
fn escape_in_the_chooser_returns_to_the_rows_with_an_empty_query() {
    let finder = typed(with_sources(vec![row("one")], &["home"]), "o");
    let finder = typed(press(finder, &["ctrl-space"]), "h");
    let finder = press(finder, &["esc"]);

    assert_eq!(finder.listed(), ["one"]);
    assert_eq!(finder.query(), "");
}

#[test]
fn choosing_a_source_ends_the_step_with_that_source() {
    let finder = press(
        with_sources(vec![row("one")], &["home", "disk"]),
        &["ctrl-space", "down"],
    );

    let (finder, effects) = step(finder, Input::Key("enter".to_string()));

    assert!(matches!(&effects[..], [Effect::Close(found)] if found.ending == Ending::SwitchSource));
    assert_eq!(finder.view().source.as_deref(), Some("disk"));
}

#[test]
fn choosing_a_group_lists_only_its_rows() {
    let rows = vec![in_group("one", "docs"), in_group("two", "src")];
    let finder = typed(press(with_sources(rows, &[]), &["ctrl-space"]), "src");
    let finder = press(finder, &["enter"]);

    assert_eq!(finder.listed(), ["two"]);
    assert_eq!(
        finder.view().view,
        View::List {
            group: Some("src".to_string())
        }
    );
}

#[test]
fn alt_o_sends_the_chosen_rows() {
    let finder = press(finder_with(&["one", "two"]), &["down"]);

    let found = found_by(finder, "alt-o");

    assert_eq!((found.ending, found.rows), (Ending::SendRows, vec![1]));
}

#[test]
fn ctrl_q_saves_the_marked_rows_as_the_quickfix_list() {
    let rows = vec![editing("one", "a.rs"), editing("two", "b.rs")];
    let finder = press(finder_of(rows.clone()), &["tab", "tab"]);

    let effects = effects_of(finder, "ctrl-q");

    let entries: Vec<QuickfixEntry> = quickfix_entries(&[&rows[0], &rows[1]]);
    assert_eq!(effects[0], Effect::SaveQuickfix(entries));
    assert!(matches!(&effects[1], Effect::Close(found) if found.ending == Ending::SaveQuickfix));
}

#[test]
fn ctrl_q_without_file_locations_keeps_the_finder_open_and_says_why() {
    let finder = press(finder_with(&["one"]), &["ctrl-q"]);

    assert_eq!(
        finder.remark(),
        Some(&Remark::Refused(
            "No file locations in the selection.".to_string()
        ))
    );
}

#[test]
fn f1_shows_help_until_the_cursor_moves() {
    let finder = press(finder_with(&["one", "two"]), &["f1"]);
    assert!(finder.help_shown());

    assert!(!press(finder, &["down"]).help_shown());
}

#[test]
fn copy_and_stay_keeps_the_finder_open_and_reports_copied() {
    let finder = finder_of(vec![claiming(
        "one",
        "alt-y",
        Action::CopyStay("one".into()),
    )]);

    let (finder, effects) = step(finder, Input::Key("alt-y".to_string()));

    assert_eq!(effects, [Effect::Copy("one".to_string()), Effect::Draw]);
    assert_eq!(finder.remark(), Some(&Remark::Done("Copied".to_string())));
}

#[test]
fn a_key_the_marked_rows_disagree_on_keeps_the_finder_open_and_says_why() {
    let rows = vec![
        claiming("one", "ctrl-e", Action::Run("ls".into())),
        row("two"),
    ];
    let finder = press(finder_of(rows), &["tab", "tab", "ctrl-e"]);

    assert_eq!(
        finder.remark(),
        Some(&Remark::Refused(
            "The marked rows do different things on ctrl-e".to_string()
        ))
    );
}
