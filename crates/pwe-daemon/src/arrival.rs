use std::sync::mpsc::Sender;

use pwe_protocol::{RexEvent, RexLink};

use crate::core::Answer;

pub(crate) const WAITING_ARRIVALS: usize = 256;

pub(crate) struct Request {
    pub(crate) line: String,
    pub(crate) answer_to: Sender<Answer>,
}

pub(crate) enum Arrival {
    Request(Request),
    Rex(RexNews),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RexNews {
    Linked(RexLink),
    Event(RexEvent),
}
