use std::io::Write;
use std::time::Duration;

use herdr_picker_domain::{Action, Key, PickerRow, Preview};
use serde_json::{Value, json};

use super::automation::automation;
use super::page::{fields, notice, own_program, words};
use super::paint::dim;
use super::rows::{RowWriter, key};
use super::written;
use crate::command::{CommandError, failed};

const SECTIONS: [(&str, &str); 3] = [
    ("topApp", "Favorites"),
    ("pinned", "Pinned"),
    ("unpinned", "Unpinned"),
];

// Arc's open tabs, or Chrome's once Alt-B switched to it.
pub(super) fn tabs(
    state: Option<&str>,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let chrome = state.is_some_and(|state| state == r#""chrome""#);
    let browser = if chrome { "chrome" } else { "arc" };
    let listed = match cfg!(target_os = "macos") {
        false => vec![notice(
            "Arc tabs need macOS",
            "Arc and Chrome are read through AppleScript, which only macOS has.",
        )],
        true => match automation(
            &json!({"action": "tabs", "browser": browser}),
            Duration::from_secs(10),
        ) {
            Ok(Value::Array(tabs)) => tab_rows(&tabs, browser, &own_program()),
            Ok(_) => vec![unanswered("osascript printed no list of tabs", browser)],
            Err(why) => vec![unanswered(&why, browser)],
        },
    };
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

pub(super) fn tab_rows(tabs: &[Value], browser: &str, program: &str) -> Vec<PickerRow> {
    let other = other(browser);
    tabs.iter()
        .map(|tab| {
            let said = |field: &str| tab[field].as_str().unwrap_or_default();
            let (title, url) = (said("title"), said("url"));
            let named = if title.is_empty() { url } else { title };
            let space = tab["space"]["title"].as_str().unwrap_or_default();
            let location = said("location");
            let section = SECTIONS
                .iter()
                .find(|(code, _)| *code == location)
                .map_or(location, |(_, section)| section);
            let details = [
                ("URL", url.to_string()),
                ("Space", space.to_string()),
                ("Section", section.to_string()),
                ("Browser", shown(browser).to_string()),
            ];
            let mut focus = vec!["tabs", "focus", browser, said("window"), said("tab")];
            focus.extend(tab["space"]["id"].as_str());
            let site = site(url);
            PickerRow {
                label: match site.is_empty() {
                    true => named.to_string(),
                    false => format!("{named}  {}", dim(site)),
                },
                preview: Some(Preview::Text(format!(
                    "{named}\n\n{}\n\nEnter switches to this tab, Alt-Y copies the URL, Alt-B switches to {}.",
                    fields(&details),
                    shown(other)
                ))),
                enter: Action::Run(format!("{program} {}", words(&focus))),
                keys: vec![
                    (key("alt-y"), Action::CopyStay(url.to_string())),
                    switch(browser),
                ],
                group: Some(space.to_string()).filter(|space| !space.is_empty()),
            }
        })
        .collect()
}

// The host a URL names, without a leading www.
fn site(url: &str) -> &str {
    let Some((_, rest)) = url.split_once("://") else {
        return "";
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = host.split(':').next().unwrap_or_default();
    host.strip_prefix("www.").unwrap_or(host)
}

pub(super) fn unanswered(why: &str, browser: &str) -> PickerRow {
    let shown = shown(browser);
    let label = match why {
        _ if why.ends_with("is not running") => format!("{shown} is not running"),
        _ if why.starts_with("Application can't be found") => format!("{shown} is not installed"),
        _ => format!("{shown} did not answer"),
    };
    PickerRow {
        keys: vec![switch(browser)],
        ..notice(&label, why)
    }
}

fn other(browser: &str) -> &'static str {
    if browser == "chrome" { "arc" } else { "chrome" }
}

fn shown(browser: &str) -> &'static str {
    if browser == "chrome" { "Chrome" } else { "Arc" }
}

// Alt-B reads the other browser's tabs.
fn switch(browser: &str) -> (Key, Action) {
    (
        key("alt-b").labelled(&format!("{} tabs", shown(other(browser)))),
        Action::State(json!(other(browser)).to_string()),
    )
}

// herdr-picker tabs focus <browser> <window> <tab> [space]
pub(crate) fn tabs_tool(arguments: &[&str]) -> Result<(), CommandError> {
    let ["focus", browser, window, tab, space @ ..] = arguments else {
        return Err(CommandError::Usage);
    };
    let item =
        json!({"window": window, "tab": tab, "space": space.first().map(|id| json!({"id": id}))});
    let request = json!({"action": "focus", "browser": browser, "item": item});
    automation(&request, Duration::from_secs(10))
        .map(|_| ())
        .map_err(failed)
}

#[cfg(test)]
mod tests;
