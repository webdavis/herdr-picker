use super::BOLD_BLUE;
use super::paint::Paint;
use super::row::{BEFORE_LABEL, Columns};
use super::text::{ELLIPSIS, fill, fit, room};

pub(super) const LABEL_INDENT: &str = " ";

pub(super) fn padded(label: &str, columns: Columns, width: usize) -> String {
    let name = fit(&format!("{LABEL_INDENT}{label}"), room(width, BEFORE_LABEL));
    let column = room(width, BEFORE_LABEL).map_or(columns.name_column(), |left| {
        columns.name_column().min(left)
    });
    fill(&name, column)
}

pub(super) fn lit(name: &str, matched: &[usize], paint: &Paint) -> String {
    let mut drawn = String::new();
    let mut run = String::new();
    let mut run_lit = false;
    for (index, character) in name.chars().enumerate() {
        let lit = character != ELLIPSIS
            && index
                .checked_sub(LABEL_INDENT.len())
                .is_some_and(|at| matched.contains(&at));
        if lit != run_lit {
            drawn.push_str(&paint.cell(style(run_lit), &run));
            run.clear();
            run_lit = lit;
        }
        run.push(character);
    }
    drawn.push_str(&paint.cell(style(run_lit), &run));
    drawn
}

fn style(lit: bool) -> &'static str {
    if lit { BOLD_BLUE } else { "" }
}
