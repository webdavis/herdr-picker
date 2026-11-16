use std::path::PathBuf;

pub(crate) fn expand_tilde(raw: &str, home: Option<&str>) -> Result<PathBuf, String> {
    let tail = match raw.strip_prefix('~') {
        None => return Ok(PathBuf::from(raw)),
        Some(tail) => tail,
    };
    if !tail.is_empty() && !tail.starts_with('/') {
        return Err(format!("cannot expand another user's home in {raw}"));
    }
    let home = home
        .filter(|home| !home.is_empty())
        .ok_or_else(|| format!("cannot expand {raw}: HOME is unset"))?;
    Ok(PathBuf::from(home).join(tail.trim_start_matches('/')))
}

#[cfg(test)]
mod tests;
