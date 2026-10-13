use super::{GREY_BAR, RESET};

pub(super) struct Paint {
    pub(super) bar: &'static str,
}

impl Paint {
    pub(super) fn plain() -> Self {
        Self { bar: "" }
    }

    pub(super) fn for_row(selected: bool) -> Self {
        Self {
            bar: if selected { GREY_BAR } else { "" },
        }
    }

    pub(super) fn cell(&self, style: &str, text: &str) -> String {
        if text.is_empty() || (self.bar.is_empty() && style.is_empty()) {
            text.to_string()
        } else {
            format!("{}{style}{text}{RESET}", self.bar)
        }
    }
}
