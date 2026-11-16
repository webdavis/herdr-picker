use pwe_domain::finder::RowMatcher;

use super::*;

fn kept<'a>(query: &str, labels: &[&'a str]) -> Vec<&'a str> {
    let rows: Vec<String> = labels.iter().map(|label| label.to_string()).collect();
    FrizbeeMatcher
        .rank(query, &rows)
        .into_iter()
        .map(|row| labels[row])
        .collect()
}

#[test]
fn an_apostrophe_word_keeps_only_rows_with_that_exact_text() {
    assert_eq!(
        kept("'ram", &["program", "r-a-m", "ramp"]),
        ["ramp", "program"]
    );
}

#[test]
fn a_plain_word_keeps_rows_with_its_letters_in_order() {
    assert_eq!(kept("rm", &["readme", "mr", "rim"]), ["rim", "readme"]);
}

#[test]
fn a_caret_word_keeps_rows_that_start_with_it() {
    assert_eq!(
        kept("^src", &["src/main.rs", "lib/src.rs"]),
        ["src/main.rs"]
    );
}

#[test]
fn a_dollar_word_keeps_rows_that_end_with_it() {
    assert_eq!(kept(".rs$", &["main.rs", "main.rsx"]), ["main.rs"]);
}

#[test]
fn a_bang_word_drops_rows_that_contain_it() {
    assert_eq!(kept("!test", &["main.rs", "tests.rs"]), ["main.rs"]);
}

#[test]
fn every_word_must_match() {
    assert_eq!(
        kept("src main", &["src/main.rs", "src/lib.rs", "main.rs"]),
        ["src/main.rs"]
    );
}

#[test]
fn a_lower_case_query_ignores_case_and_an_upper_case_one_respects_it() {
    assert_eq!(kept("readme", &["README", "readme"]).len(), 2);
    assert_eq!(kept("README", &["README", "readme"]), ["README"]);
}

#[test]
fn an_empty_query_keeps_every_row_in_arrival_order() {
    assert_eq!(kept("", &["b", "a", "c"]), ["b", "a", "c"]);
}

#[test]
fn highlight_positions_are_characters_in_reading_order() {
    assert_eq!(FrizbeeMatcher.highlight("abc", "é-abc"), [2, 3, 4]);
    assert_eq!(FrizbeeMatcher.highlight("ma rs", "main.rs"), [0, 1, 5, 6]);
}

#[test]
fn a_row_the_query_does_not_match_has_no_highlight() {
    assert_eq!(
        FrizbeeMatcher.highlight("xyz", "main.rs"),
        Vec::<usize>::new()
    );
}
