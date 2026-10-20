use std::path::Path;

use herdr_picker_adapters::{
    change_footer, copy_to_clipboard, error_footer, finder_actions, is_live_step, leave,
    read_run_error, read_view, unbound_action, write_view,
};
use herdr_picker_domain::{
    Action, Key, PickerRow, Theme, ViewKey, ViewResponse, ViewState, act, groups, press,
};

use crate::command::{CommandError, failed};
use crate::picker_chrome::{QuotedHelper, rows_of};

// fzf runs this for every key the picker binds; its output is the fzf actions that answer it.
pub(crate) fn view_key(cache: &Path, event: &[&str], theme: Theme) -> Result<String, CommandError> {
    let answer = answer(cache, event, theme)?;
    let unclaimed = match event {
        ["key", name, ..] if answer.is_empty() => unbound_action(name),
        _ => None,
    };
    Ok(unclaimed.map_or(answer, str::to_string))
}

fn answer(cache: &Path, event: &[&str], theme: Theme) -> Result<String, CommandError> {
    let (view_key, pressed) = match event {
        ["load"] => (ViewKey::Loaded, None),
        ["choose"] => (ViewKey::Choose, None),
        ["back"] => (ViewKey::Back, None),
        ["key", name, fields @ ..] => {
            let key = Key::parse(name).ok_or(CommandError::Usage)?;
            let marked = indices(fields);
            let view_key = match key == Key::enter() {
                true => ViewKey::Enter(marked.first().copied()),
                false => ViewKey::Other,
            };
            (view_key, Some((key, marked)))
        }
        _ => return Err(CommandError::Usage),
    };
    let rows = rows_of(cache)?;
    let state = read_view(cache);
    let (view, response) = press(&state.view, view_key, &state.sources, &groups(&rows));
    let source = match &response {
        ViewResponse::Switch(source) => Some(source.clone()),
        _ => state.source.clone(),
    };
    write_view(
        cache,
        &ViewState {
            view,
            source,
            ..state
        },
    )
    .map_err(failed)?;
    if let (ViewResponse::Accept, Some((key, marked))) = (&response, &pressed) {
        return Ok(act_on(&rows, marked, key, theme));
    }
    let failed_run = view_key == ViewKey::Loaded && read_run_error(cache).is_some();
    let quoted = QuotedHelper::for_run(cache)?;
    Ok(finder_actions(
        response,
        quoted.helper(),
        failed_run,
        is_live_step(cache),
    ))
}

// fzf hands the marked rows' indexes as separate words, or the highlighted row's when none is.
pub(crate) fn indices(fields: &[&str]) -> Vec<usize> {
    fields
        .iter()
        .flat_map(|field| field.split_whitespace())
        .filter_map(|field| field.parse().ok())
        .collect()
}

// A key that leaves closes fzf, and the run acts once it has the query and marked rows; copy and
// stay is the one action done here, since fzf stays open for it.
fn act_on(rows: &[PickerRow], marked: &[usize], key: &Key, theme: Theme) -> String {
    let marked: Vec<&PickerRow> = marked.iter().filter_map(|index| rows.get(*index)).collect();
    let refused = |reason: &str| change_footer(&error_footer(reason, usize::MAX, theme));
    match act(&marked, key) {
        Err(refusal) => refused(&refusal.to_string()),
        Ok(Action::Nothing) => String::new(),
        Ok(Action::CopyStay(text)) => match copy_to_clipboard(&text) {
            Ok(()) => change_footer("Copied"),
            Err(failure) => refused(&format!("Cannot copy: {failure}")),
        },
        Ok(_) => leave(key.name()),
    }
}

#[cfg(test)]
mod tests;
