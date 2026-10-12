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
