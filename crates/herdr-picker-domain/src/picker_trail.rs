use crate::picker_header::title;
use crate::picker_row::{Action, PickerCall};

// What a step shows: a picker's rows, or a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Picker(PickerCall),
    Confirm { question: String, action: Action },
}

// Where the user was on a step: restored when Esc comes back to it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spot {
    pub query: String,
    pub group: Option<String>,
    // None reads the picker's first source.
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub page: Page,
    pub spot: Spot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Turn {
    Show,
    Leave(Action),
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trail {
    // Never empty: the last step is the one showing.
    steps: Vec<Step>,
}

impl Trail {
    pub fn new(first: PickerCall) -> Self {
        Self {
            steps: vec![step(Page::Picker(first))],
        }
    }

    pub fn current(&self) -> &Step {
        self.steps.last().expect("a trail keeps its first step")
    }

    pub fn titles(&self) -> Vec<String> {
        self.steps
            .iter()
            .map(|step| match &step.page {
                Page::Picker(call) => title(&call.name),
                Page::Confirm { question, .. } => question.clone(),
            })
            .collect()
    }

    pub fn advance(&mut self, action: Action, spot: Spot) -> Turn {
        let answered =
            matches!(self.current().page, Page::Confirm { .. }) && action != Action::Back;
        if answered {
            self.steps.pop();
        } else if let Some(current) = self.steps.last_mut() {
            current.spot = spot;
        }
        match action {
            Action::Next(call) => {
                let query = call.query.clone().unwrap_or_default();
                // Going deeper into the same picker, such as a level down, keeps the source in view.
                let source = match &self.current().page {
                    Page::Picker(current) if current.name == call.name => {
                        self.current().spot.source.clone()
                    }
                    _ => None,
                };
                let turn = self.push(Page::Picker(call));
                if let Some(opened) = self.steps.last_mut() {
                    opened.spot.query = query;
                    opened.spot.source = source;
                }
                turn
            }
            Action::Confirm { question, action } => self.push(Page::Confirm {
                question,
                action: *action,
            }),
            Action::Back if self.steps.len() > 1 => {
                self.steps.pop();
                Turn::Show
            }
            Action::Back => Turn::Close,
            Action::Source(source) => {
                if let Some(current) = self.steps.last_mut() {
                    current.spot.source = Some(source);
                    current.spot.group = None;
                }
                Turn::Show
            }
            Action::State(state) => {
                if let Some(Page::Picker(call)) = self.steps.last_mut().map(|step| &mut step.page) {
                    call.state = Some(state);
                }
                Turn::Show
            }
            Action::Reload | Action::Nothing | Action::CopyStay(_) => Turn::Show,
            leaving => Turn::Leave(leaving),
        }
    }

    fn push(&mut self, page: Page) -> Turn {
        self.steps.push(step(page));
        Turn::Show
    }
}

fn step(page: Page) -> Step {
    Step {
        page,
        spot: Spot::default(),
    }
}

#[cfg(test)]
mod tests;
