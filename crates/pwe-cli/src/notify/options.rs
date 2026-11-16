use pwe_domain::Level;
use pwe_protocol::{NoticeRequest, NotifyRequest};

use crate::command::CommandError;

const DEFAULT_KIND: &str = "notice";

#[derive(Default)]
struct Flags<'a> {
    source: Option<&'a str>,
    level: Option<&'a str>,
    kind: Option<&'a str>,
    key: Option<&'a str>,
    pane: Option<&'a str>,
    text: Option<&'a str>,
}

pub(crate) fn request(options: &[&str], now: u64, pid: u32) -> Result<NoticeRequest, CommandError> {
    if let ["resolve", "--key", key] = options {
        return Ok(NoticeRequest::Resolve {
            key: key.to_string(),
        });
    }
    let flags = flags(options)?;
    let (Some(source), Some(level), Some(text)) = (flags.source, flags.level, flags.text) else {
        return Err(CommandError::Usage);
    };
    let level = Level::parse(level).ok_or_else(|| {
        CommandError::Failed(format!(
            "--level does not accept \"{level}\"; it accepts needs-you, finished, quiet"
        ))
    })?;
    Ok(NoticeRequest::Notify(NotifyRequest {
        source: source.to_string(),
        kind: flags.kind.unwrap_or(DEFAULT_KIND).to_string(),
        key: flags
            .key
            .map_or_else(|| format!("{source}-{now}-{pid}"), str::to_string),
        level,
        text: text.to_string(),
        pane: flags.pane.map(str::to_string),
    }))
}

fn flags<'a>(options: &[&'a str]) -> Result<Flags<'a>, CommandError> {
    let mut flags = Flags::default();
    let mut rest = options;
    while let [first, after @ ..] = rest {
        let slot = match *first {
            "--source" => &mut flags.source,
            "--level" => &mut flags.level,
            "--kind" => &mut flags.kind,
            "--key" => &mut flags.key,
            "--pane" => &mut flags.pane,
            text if !text.starts_with("--") && flags.text.is_none() => {
                flags.text = Some(text);
                rest = after;
                continue;
            }
            _ => return Err(CommandError::Usage),
        };
        let [value, after @ ..] = after else {
            return Err(CommandError::Usage);
        };
        *slot = Some(value);
        rest = after;
    }
    Ok(flags)
}
