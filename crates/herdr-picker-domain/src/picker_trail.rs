use crate::picker_row::{Action, PickerCall};
use crate::picker_view::title;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Picker(PickerCall),
    Confirm { question: String, action: Action },
}

// Where the user was on a step: restored when Esc comes back to it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spot {
    pub query: String,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub source: Source,
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
            steps: vec![step(Source::Picker(first))],
        }
    }

    pub fn current(&self) -> &Step {
        self.steps.last().expect("a trail keeps its first step")
    }

    pub fn titles(&self) -> Vec<String> {
        self.steps
            .iter()
            .map(|step| match &step.source {
                Source::Picker(call) => title(&call.name),
                Source::Confirm { question, .. } => question.clone(),
            })
            .collect()
    }

    pub fn advance(&mut self, action: Action, spot: Spot) -> Turn {
        let answered =
            matches!(self.current().source, Source::Confirm { .. }) && action != Action::Back;
        if answered {
            self.steps.pop();
        } else if let Some(current) = self.steps.last_mut() {
            current.spot = spot;
        }
        match action {
            Action::Next(call) => self.push(Source::Picker(call)),
            Action::Confirm { question, action } => self.push(Source::Confirm {
                question,
                action: *action,
            }),
            Action::Back if self.steps.len() > 1 => {
                self.steps.pop();
                Turn::Show
            }
            Action::Back => Turn::Close,
            Action::Reload | Action::Nothing | Action::CopyStay(_) => Turn::Show,
            leaving => Turn::Leave(leaving),
        }
    }

    fn push(&mut self, source: Source) -> Turn {
        self.steps.push(step(source));
        Turn::Show
    }
}

fn step(source: Source) -> Step {
    Step {
        source,
        spot: Spot::default(),
    }
}

#[cfg(test)]
mod tests;
