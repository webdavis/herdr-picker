use super::text::width_of;
use super::{ALREADY_CURRENT, MARGIN};
use herdr_workspace_jump_domain::{JumpTarget, Legend, RowLook};

const KEY_COLUMN_WIDTH: usize = 4;
pub(super) const LABEL_INDENT: &str = "  ";
pub(super) const FILLER_AROUND_LEADERS: usize = 3;
pub(super) const BEFORE_LABEL: usize = KEY_COLUMN_WIDTH + LABEL_INDENT.len();

#[derive(Clone, Debug)]
pub struct Layout {
    legend: Legend,
    look: RowLook,
    labels: Vec<String>,
    hidden: Option<String>,
    notice: Option<&'static str>,
}

impl Layout {
    pub fn new(legend: Legend, targets: &[JumpTarget]) -> Self {
        Self {
            legend,
            look: RowLook::default(),
            labels: targets.iter().map(|target| target.label.clone()).collect(),
            hidden: None,
            notice: None,
        }
    }

    pub fn look(self, look: RowLook) -> Self {
        Self { look, ..self }
    }

    pub fn hiding(self, label: Option<&str>) -> Self {
        Self {
            hidden: label.map(str::to_string),
            ..self
        }
    }

    pub fn noting(self, notice: &'static str) -> Self {
        Self {
            notice: Some(notice),
            ..self
        }
    }

    pub fn columns_to_fit(&self) -> usize {
        let widest = self.labels.iter().map(|label| width_of(label)).max();
        let refusal = MARGIN.len() + width_of(ALREADY_CURRENT) + widest.unwrap_or_default();
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

    pub(super) fn row_look(&self) -> RowLook {
        self.look
    }

    pub fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    pub(super) fn directory_start(&self) -> usize {
        BEFORE_LABEL + self.widest_offered() + FILLER_AROUND_LEADERS
    }

    fn widest_offered(&self) -> usize {
        self.labels
            .iter()
            .filter(|label| Some(label.as_str()) != self.hidden.as_deref())
            .map(|label| width_of(label))
            .max()
            .unwrap_or_default()
    }
}

fn line_width(text: &str) -> usize {
    MARGIN.len() + width_of(text)
}
