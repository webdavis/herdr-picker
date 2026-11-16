use pwe_domain::finder::{Finder, Remark, RowMatcher};
use pwe_domain::{Theme, footer_hint, groups, header, help_entries};

use crate::picker_text::{error_footer, help_page, step_footer};

const PROMPT: &str = " /";

pub(super) fn prompt<M: RowMatcher>(finder: &Finder<M>) -> (String, String) {
    (
        format!("{PROMPT}{}", finder.query()),
        format!("{} ", finder.match_count()),
    )
}

pub(super) fn cursor_column<M: RowMatcher>(finder: &Finder<M>) -> u16 {
    let column = PROMPT.chars().count() + finder.query_cursor();
    u16::try_from(column).unwrap_or(u16::MAX)
}

pub(super) fn title<M: RowMatcher>(finder: &Finder<M>) -> String {
    let state = finder.view();
    let title = header(
        &state.titles,
        &state.view,
        state.source.as_deref(),
        &groups(finder.rows()),
    );
    match finder.header_line() {
        Some(line) => format!("{title}\n{line}"),
        None => title,
    }
}

pub(super) fn footer<M: RowMatcher>(finder: &Finder<M>, width: usize, theme: Theme) -> String {
    match finder.remark() {
        Some(Remark::Done(text)) => text.clone(),
        Some(Remark::Refused(text)) => error_footer(text, width, theme),
        None => {
            let hint = finder.current_row().map(footer_hint).unwrap_or_default();
            let error = finder.run_error().unwrap_or_default();
            step_footer(&hint, &finder.view().description, error, width, theme)
        }
    }
}

pub(super) fn help<M: RowMatcher>(finder: &Finder<M>, title: &str, theme: Theme) -> String {
    let state = finder.view();
    let grouped = !groups(finder.rows()).is_empty() || !state.sources.is_empty();
    help_page(title, &help_entries(finder.current_row(), grouped), theme)
}
