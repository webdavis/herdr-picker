use std::path::Path;

const TEST_PARTS: [&str; 6] = ["test", "tests", "spec", "specs", "__tests__", "fixtures"];
const CONFIG_SUFFIXES: [&str; 8] = ["toml", "yaml", "yml", "json", "ini", "cfg", "plist", "lock"];
// Where a file sits in a project: its tests, docs, configuration or the source itself.
pub(super) fn area(name: &str) -> &'static str {
    let path = Path::new(name);
    let parts: Vec<String> = name.split('/').map(str::to_lowercase).collect();
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let suffix = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let file = path.file_name().unwrap_or_default().to_string_lossy();
    if parts.iter().any(|part| TEST_PARTS.contains(&part.as_str())) || stem.contains("test") {
        return "tests";
    }
    if parts.iter().any(|part| part == "doc" || part == "docs")
        || ["md", "rst", "txt"].contains(&suffix.as_str())
    {
        return "docs";
    }
    if CONFIG_SUFFIXES.contains(&suffix.as_str()) || file.starts_with('.') {
        return "config";
    }
    "source"
}
