use std::mem;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use pwe_domain::PickerRow;
use pwe_domain::finder::Input;

const RANK_AT_MOST_EVERY: Duration = Duration::from_millis(50);

pub struct Arrivals {
    inputs: Receiver<Input>,
    waiting: Vec<PickerRow>,
    ranked_at: Option<Instant>,
}

impl Arrivals {
    pub fn new(inputs: Receiver<Input>) -> Self {
        Self {
            inputs,
            waiting: Vec::new(),
            ranked_at: None,
        }
    }

    pub fn ready(&mut self, now: Instant) -> Vec<Input> {
        let mut ready = Vec::new();
        while let Ok(input) = self.inputs.try_recv() {
            match input {
                Input::RowsArrived(rows) => self.waiting.extend(rows),
                preview @ Input::PreviewArrived { .. } => ready.push(preview),
                after_the_rows => {
                    ready.extend(self.release(now));
                    ready.push(after_the_rows);
                }
            }
        }
        let due = self
            .ranked_at
            .is_none_or(|ranked_at| now.duration_since(ranked_at) >= RANK_AT_MOST_EVERY);
        if due {
            ready.extend(self.release(now));
        }
        ready
    }

    fn release(&mut self, now: Instant) -> Option<Input> {
        if self.waiting.is_empty() {
            return None;
        }
        self.ranked_at = Some(now);
        Some(Input::RowsArrived(mem::take(&mut self.waiting)))
    }
}

#[cfg(test)]
mod tests;
