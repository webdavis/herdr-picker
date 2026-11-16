use std::sync::mpsc::{Receiver, SyncSender};

use pwe_adapters::RexEventStream;

use crate::agents::{AgentReader, ReadAgents};
use crate::arrival::Arrival;
use crate::rex_stream::RexFollower;

pub(crate) struct AgentsToRead {
    pub(crate) read: ReadAgents,
    pub(crate) refresh_seconds: u64,
    pub(crate) asked: Receiver<()>,
}

#[derive(Default)]
pub(crate) struct Companions {
    pub(crate) rex: Option<RexEventStream>,
    pub(crate) agents: Option<AgentsToRead>,
}

pub(crate) struct RunningCompanions {
    follower: Option<RexFollower>,
    reader: Option<AgentReader>,
}

impl Companions {
    pub(crate) fn start(self, arrivals: &SyncSender<Arrival>) -> RunningCompanions {
        RunningCompanions {
            follower: self
                .rex
                .map(|stream| RexFollower::start(stream, arrivals.clone())),
            reader: self.agents.map(|agents| {
                AgentReader::start(
                    agents.read,
                    agents.refresh_seconds,
                    agents.asked,
                    arrivals.clone(),
                )
            }),
        }
    }
}

impl RunningCompanions {
    pub(crate) fn stop(self) {
        if let Some(follower) = self.follower {
            follower.stop();
        }
        if let Some(reader) = self.reader {
            reader.stop();
        }
    }
}
