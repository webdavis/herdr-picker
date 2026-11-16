pub(super) fn swift_test(line: &str) -> bool {
    let Some((owner, test)) = line.rsplit_once('/') else {
        return false;
    };
    let dotted = owner
        .split_once('.')
        .is_some_and(|(module, rest)| !module.is_empty() && !rest.is_empty());
    dotted && !test.is_empty() && !line.contains(char::is_whitespace)
}

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

pub(super) fn escaped(test: &str) -> String {
    test.chars()
        .flat_map(|character| match r"\.+*?()|[]{}^$".contains(character) {
            true => vec!['\\', character],
            false => vec![character],
        })
        .collect()
}
