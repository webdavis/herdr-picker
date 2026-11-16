// `Module.Type/test` with no spaces, as `swift test list` prints each test.
pub(super) fn swift_test(line: &str) -> bool {
    let Some((owner, test)) = line.rsplit_once('/') else {
        return false;
    };
    let dotted = owner
        .split_once('.')
        .is_some_and(|(module, rest)| !module.is_empty() && !rest.is_empty());
    dotted && !test.is_empty() && !line.contains(char::is_whitespace)
}

// What `go test -list` prints for each test, example and fuzz target.
pub(super) fn go_test(line: &str) -> bool {
    let word = |rest: &str| {
        rest.chars()
            .all(|character| character.is_alphanumeric() || character == '_')
    };
    ["Test", "Fuzz"].iter().any(|prefix| {
        line.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && word(rest))
    }) || line.strip_prefix("Example").is_some_and(word)
}

// The test's name as a regular expression that matches only it.
pub(super) fn escaped(test: &str) -> String {
    test.chars()
        .flat_map(|character| match r"\.+*?()|[]{}^$".contains(character) {
            true => vec!['\\', character],
            false => vec![character],
        })
        .collect()
}
