use std::fs;
use std::path::Path;

use herdr_picker_domain::{Place, remember};

// One line per place, oldest first: seconds, workspace, tab, pane and folder, split by tabs.
// shortcut: two focus hooks writing at once can lose one place; lock the file if `recent` misses them.
pub fn record_place(path: &Path, place: Place, bound: usize) {
    if place.cwd.contains(['\t', '\n']) {
        return;
    }
    let places = remember(read(path), place, bound);
    let lines: String = places.iter().map(line).collect();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, lines);
}

pub fn recent_places(path: &Path) -> Vec<Place> {
    let mut places = read(path);
    places.reverse();
    places
}

fn read(path: &Path) -> Vec<Place> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(parse)
        .collect()
}

fn line(place: &Place) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\n",
        place.at, place.workspace_id, place.tab_id, place.pane_id, place.cwd
    )
}

fn parse(line: &str) -> Option<Place> {
    let mut fields = line.splitn(5, '\t');
    Some(Place {
        at: fields.next()?.parse().ok()?,
        workspace_id: fields.next()?.to_string(),
        tab_id: fields.next()?.to_string(),
        pane_id: fields.next()?.to_string(),
        cwd: fields.next()?.to_string(),
    })
}

#[cfg(test)]
mod tests;
