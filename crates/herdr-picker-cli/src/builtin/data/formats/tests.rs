use std::path::Path;

use super::*;

#[test]
fn a_data_file_is_known_by_its_extension() {
    let format = |name: &str| Format::of(Path::new(name));

    assert_eq!(format("a.json"), Some(Format::Json));
    assert_eq!(format("logs/run.jsonl"), Some(Format::Lines));
    assert_eq!(format("run.NDJSON"), Some(Format::Lines));
    assert_eq!(format("compose.yml"), Some(Format::Yaml));
    assert_eq!(format("ci.yaml"), Some(Format::Yaml));
    assert_eq!(format("users.toon"), Some(Format::Toon));
    assert_eq!(format("app.sqlite"), Some(Format::Sqlite));
    assert_eq!(format("app.db"), Some(Format::Sqlite));
    assert_eq!(format("notes.txt"), None);
    assert_eq!(format("json"), None);
}

#[test]
fn yaml_and_toon_are_read_as_json_through_yq_and_toon() {
    let file = Path::new("/w/a b.yaml");

    assert_eq!(
        Format::Yaml.reader(file),
        Some(
            ["yq", "-o", "json", "-I0", "/w/a b.yaml"]
                .iter()
                .map(|w| w.to_string())
                .collect()
        )
    );
    assert_eq!(
        Format::Toon.reader(Path::new("/w/u.toon")),
        Some(vec![
            "toon".to_string(),
            "--decode".to_string(),
            "/w/u.toon".to_string()
        ])
    );
    assert_eq!(Format::Json.reader(file), None);
}

#[test]
fn a_copy_is_converted_from_json_by_yq_or_toon_and_json_needs_no_program() {
    assert_eq!(Output::Json.converter(), None);
    assert_eq!(
        Output::Yaml.converter(),
        Some(&["yq", "-p", "json", "-o", "yaml", "-"][..])
    );
    assert_eq!(Output::Toon.converter(), Some(&["toon", "--encode"][..]));
}
