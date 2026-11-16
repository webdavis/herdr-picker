pub fn variable_in_ps_listing(printed: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    printed
        .lines()
        .skip(1)
        .flat_map(str::split_whitespace)
        .find_map(|word| word.strip_prefix(&prefix))
        .map(str::to_string)
}

pub fn variable_in_environ_file(contents: &[u8], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    contents
        .split(|byte| *byte == 0)
        .find_map(|entry| entry.strip_prefix(prefix.as_bytes()))
        .map(|value| String::from_utf8_lossy(value).into_owned())
}

#[cfg(test)]
mod tests;
