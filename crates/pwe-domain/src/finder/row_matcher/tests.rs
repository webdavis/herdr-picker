use super::*;

fn rows(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|label| label.to_string()).collect()
}

#[test]
fn fuzzy_ranks_row_numbers_best_first() {
    let ranked = RowMatcher::rank(&Fuzzy, "abc", &rows(&["a-b-c", "zzz", "abc"]));

    assert_eq!(ranked, [2, 0]);
}

#[test]
fn fuzzy_highlights_the_characters_it_matched() {
    assert_eq!(RowMatcher::highlight(&Fuzzy, "ac", "abc"), [0, 2]);
    assert_eq!(
        RowMatcher::highlight(&Fuzzy, "x", "abc"),
        Vec::<usize>::new()
    );
}
