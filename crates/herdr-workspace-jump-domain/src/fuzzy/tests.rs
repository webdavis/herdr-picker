use super::*;

fn order<'a>(query: &str, labels: &[&'a str]) -> Vec<&'a str> {
    Fuzzy
        .rank(query, labels, |label| label)
        .into_iter()
        .copied()
        .collect()
}

#[test]
fn a_label_matches_when_it_holds_the_letters_in_order_ignoring_case() {
    assert_eq!(order("dot", &["dotfiles", "Ivy", "tod"]), ["dotfiles"]);
    assert_eq!(order("DOT", &["dotfiles"]), ["dotfiles"]);
    assert_eq!(order("hlb", &["homelab"]), ["homelab"]);
    assert!(order("zz", &["dotfiles", "homelab"]).is_empty());
}

#[test]
fn an_empty_query_keeps_every_label_in_the_order_given() {
    assert_eq!(
        order("", &["netpulse", "Ivy", "homelab"]),
        ["netpulse", "Ivy", "homelab"]
    );
}

#[test]
fn a_run_of_the_letters_beats_a_scatter() {
    assert_eq!(
        order("es", &["netpulse", "essential"]),
        ["essential", "netpulse"]
    );
}

#[test]
fn an_early_match_beats_a_late_one() {
    assert_eq!(
        order("es", &["fitness", "essential"]),
        ["essential", "fitness"]
    );
}

#[test]
fn among_equal_matches_the_shorter_label_wins_and_then_the_order_given() {
    assert_eq!(
        order("ho", &["homelab-old", "homelab"]),
        ["homelab", "homelab-old"]
    );
    assert_eq!(order("ab", &["abx", "aby"]), ["abx", "aby"]);
}

#[test]
fn the_tightest_of_several_spans_in_one_label_is_the_one_that_counts() {
    assert_eq!(order("pl", &["p-lantpl", "xxpl"]), ["xxpl", "p-lantpl"]);
}
