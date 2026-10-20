use super::*;

fn place(pane_id: &str, cwd: &str, at: u64) -> Place {
    Place {
        at,
        workspace_id: "w1".to_string(),
        tab_id: "w1:t1".to_string(),
        pane_id: pane_id.to_string(),
        cwd: cwd.to_string(),
    }
}

#[test]
fn a_focused_place_goes_last_and_its_earlier_visit_is_dropped() {
    let places = vec![place("p1", "/a", 1), place("p2", "/b", 2)];

    assert_eq!(
        remember(places, place("p1", "/a", 3), 10),
        vec![place("p2", "/b", 2), place("p1", "/a", 3)]
    );
}

#[test]
fn the_same_pane_in_another_folder_is_another_place() {
    let places = vec![place("p1", "/a", 1)];

    assert_eq!(
        remember(places, place("p1", "/b", 2), 10),
        vec![place("p1", "/a", 1), place("p1", "/b", 2)]
    );
}

#[test]
fn the_oldest_places_fall_off_past_the_bound() {
    let places = vec![
        place("p1", "/a", 1),
        place("p2", "/b", 2),
        place("p3", "/c", 3),
    ];

    assert_eq!(
        remember(places, place("p4", "/d", 4), 3),
        vec![
            place("p2", "/b", 2),
            place("p3", "/c", 3),
            place("p4", "/d", 4)
        ]
    );
}
