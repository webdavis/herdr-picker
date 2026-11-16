use pwe_domain::Theme;
use pwe_domain::finder::{Finder, RowMatcher};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::screen::palette;

mod chrome;
mod list;
mod styled;

use chrome::{footer, help, prompt, title};
use list::{labels, list_line};
use styled::{ansi_text, style_of};

pub fn draw<M: RowMatcher>(frame: &mut Frame, finder: &Finder<M>, theme: Theme) {
    let palette = palette(theme);
    let area = frame.area();
    let base = style_of(palette.base).patch(style_of(palette.text));
    frame.render_widget(Block::new().style(base), area);
    let (left, preview) = match finder.in_chooser() {
        true => (area, None),
        false => {
            let [left, right] = Layout::horizontal([Constraint::Fill(1); 2]).areas(area);
            (left, Some(right))
        }
    };
    let title = title(finder);
    let footer = ansi_text(&footer(finder, left.width.into(), theme));
    let [prompt_area, title_area, list_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(height_of(footer.lines.len())),
    ])
    .areas(left);
    let faint = style_of(palette.faint);
    let (query, count) = prompt(finder);
    frame.render_widget(Paragraph::new(query), prompt_area);
    frame.render_widget(
        Paragraph::new(count).style(faint).right_aligned(),
        prompt_area,
    );
    frame.render_widget(Paragraph::new(title.as_str()).style(faint), title_area);
    draw_list(frame, finder, list_area, theme);
    frame.render_widget(Paragraph::new(footer).style(faint), footer_area);
    if let Some(preview_area) = preview {
        let text = match finder.help_shown() {
            true => help(finder, &title, theme),
            false => finder.preview().unwrap_or_default().to_string(),
        };
        let border = Block::new()
            .borders(Borders::LEFT)
            .border_style(style_of(palette.leader));
        frame.render_widget(Paragraph::new(ansi_text(&text)).block(border), preview_area);
    }
}

fn draw_list<M: RowMatcher>(frame: &mut Frame, finder: &Finder<M>, area: Rect, theme: Theme) {
    let lines = finder.visible(area.height.into());
    let labels = labels(&lines);
    let rows: Vec<Rect> = (0..area.height)
        .map(|offset| Rect {
            y: area.y + offset,
            height: 1,
            ..area
        })
        .collect();
    for ((line, label), row_area) in lines.iter().zip(&labels).zip(rows) {
        frame.render_widget(list_line(line, label, theme), row_area);
    }
}

fn height_of(lines: usize) -> u16 {
    u16::try_from(lines).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
