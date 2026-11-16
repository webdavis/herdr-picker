use std::collections::HashMap;

use pwe_application::AgentSources;
use pwe_domain::Timers;
use pwe_protocol::{
    DaemonError, DaemonErrorKind, DaemonRequest, DaemonStatus, RexLink, daemon_reply_line,
    daemon_status_result, parse_daemon_request,
};
use serde_json::Value;
use serde_json::json;

use crate::agents::AgentWatch;
use crate::arrival::RexNews;

pub(crate) type Job = Box<dyn FnMut(u64) + Send>;

pub(crate) struct DaemonCore {
    status: DaemonStatus,
    timers: Timers,
    jobs: HashMap<String, Job>,
    agents: Option<AgentWatch>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Answer {
    pub(crate) reply_line: String,
    pub(crate) keeps_serving: bool,
}

impl DaemonCore {
    pub(crate) fn new(status: DaemonStatus) -> Self {
        Self {
            status,
            timers: Timers::default(),
            jobs: HashMap::new(),
            agents: None,
        }
    }

    pub(crate) fn every(&mut self, name: &str, seconds: u64, now: u64, job: Job) {
        self.timers.every(name, seconds, now);
        self.jobs.insert(name.to_string(), job);
    }

    pub(crate) fn run_due(&mut self, now: u64) {
        for name in self.timers.due(now) {
            if let Some(job) = self.jobs.get_mut(&name) {
                job(now);
            }
        }
    }

    pub(crate) fn next_at(&self) -> Option<u64> {
        self.timers.next_at()
    }

    pub(crate) fn rex_link(&mut self, link: RexLink) {
        self.status.rex = link;
    }

    pub(crate) fn watch_agents(&mut self, watch: AgentWatch) {
        self.agents = Some(watch);
    }

    pub(crate) fn hear_rex(&mut self, news: RexNews, now: u64) {
        match news {
            RexNews::Linked(link) => self.rex_link(link),
            RexNews::Event(event) => {
                if let Some(agents) = self.agents.as_mut() {
                    agents.hear_event(&event, now);
                }
            }
        }
    }

    pub(crate) fn take_agent_reading(&mut self, sources: AgentSources, now: u64) {
        if let Some(agents) = self.agents.as_mut() {
            agents.take_reading(sources, now);
        }
    }

    fn agent_listing(&self) -> Result<Value, DaemonError> {
        self.agents
            .as_ref()
            .and_then(AgentWatch::listing)
            .ok_or_else(|| DaemonError {
                kind: DaemonErrorKind::NotReadYet,
                message: "the daemon has not read the agent programs yet".to_string(),
            })
    }

    pub(crate) fn answer(&self, request_line: &str) -> Answer {
        let (reply, keeps_serving) = match parse_daemon_request(request_line) {
            Ok(DaemonRequest::Status) => (Ok(daemon_status_result(&self.status)), true),
            Ok(DaemonRequest::Stop) => (Ok(json!({ "stopping": true })), false),
            Ok(DaemonRequest::Agents) => (self.agent_listing(), true),
            Err(refusal) => (Err(refusal), true),
        };
        Answer {
            reply_line: daemon_reply_line(&reply),
            keeps_serving,
        }
    }
}

#[cfg(test)]
mod tests;
