use super::text::width_of;
use super::{ALREADY_CURRENT, MARGIN};
use herdr_workspace_jump_domain::{JumpTarget, Legend};

const KEY_COLUMN_WIDTH: usize = 4;
const MARK_COLUMN_WIDTH: usize = 4;
const GAP_BEFORE_DIRECTORY: usize = 2;
pub(super) const BEFORE_LABEL: usize = KEY_COLUMN_WIDTH + MARK_COLUMN_WIDTH;

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    legend: Legend,
    widest_label: usize,
    notice: Option<&'static str>,
}

impl Layout {
    pub fn new(legend: Legend, targets: &[JumpTarget]) -> Self {
        Self {
            legend,
            widest_label: targets
                .iter()
                .map(|target| width_of(&target.label))
                .max()
                .unwrap_or_default(),
            notice: None,
        }
    }

    pub fn noting(self, notice: &'static str) -> Self {
        Self {
            notice: Some(notice),
            ..self
        }
    }

    pub fn columns_to_fit(&self) -> usize {
        let refusal = MARGIN.len() + width_of(ALREADY_CURRENT) + self.widest_label;
        [
            refusal,
            line_width(self.legend.search),
            line_width(self.legend.keys),
        ]
        .into_iter()
        .max()
        .unwrap_or_default()
    }

    pub(super) fn legend(&self) -> Legend {
        self.legend
    }

    pub(super) fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    pub(super) fn directory_start(&self) -> usize {
        BEFORE_LABEL + self.widest_label + GAP_BEFORE_DIRECTORY
    }
}

fn line_width(text: &str) -> usize {
    MARGIN.len() + width_of(text)
}
