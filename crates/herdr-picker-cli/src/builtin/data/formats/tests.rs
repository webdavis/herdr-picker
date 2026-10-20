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
fn yaml_is_read_as_json_through_yq() {
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
    assert_eq!(Format::Json.reader(file), None);
}
