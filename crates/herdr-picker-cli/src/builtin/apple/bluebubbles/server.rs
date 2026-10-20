#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::builtin::apple) struct Server {
    pub(in crate::builtin::apple) url: String,
    // Plain HTTP is allowed only to a server on this Mac.
    pub(in crate::builtin::apple) local: bool,
}

pub(in crate::builtin::apple) fn server(url: &str) -> Result<Server, String> {
    let shape = || {
        "BlueBubbles URL must contain only its scheme, host, port, and optional base path."
            .to_string()
    };
    let (scheme, rest) = url.split_once("://").ok_or_else(shape)?;
    if rest.contains(['@', '?', '#', '"', '\\']) || rest.contains(char::is_whitespace) {
        return Err(shape());
    }
    let authority = rest.split('/').next().unwrap_or_default();
    let host = match authority.strip_prefix('[') {
        Some(bracketed) => bracketed.split(']').next().unwrap_or_default(),
        None => authority.split(':').next().unwrap_or_default(),
    };
    if host.is_empty() {
        return Err(shape());
    }
    let local = matches!(host, "localhost" | "127.0.0.1" | "::1");
    match (scheme, local) {
        ("https", _) | ("http", true) => Ok(Server {
            url: url.trim_end_matches('/').to_string(),
            local,
        }),
        _ => Err("BlueBubbles requires HTTPS outside this Mac.".to_string()),
    }
}
