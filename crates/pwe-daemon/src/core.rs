use std::collections::HashMap;

use pwe_domain::Timers;
use pwe_protocol::{
    DaemonRequest, DaemonStatus, RexLink, daemon_reply_line, daemon_status_result,
    parse_daemon_request,
};
use serde_json::json;

use crate::arrival::RexNews;

pub(crate) type Job = Box<dyn FnMut(u64) + Send>;

pub(crate) struct DaemonCore {
    status: DaemonStatus,
    timers: Timers,
    jobs: HashMap<String, Job>,
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

    pub(crate) fn hear_rex(&mut self, news: RexNews) {
        if let RexNews::Linked(link) = news {
            self.rex_link(link);
        }
    }

    pub(crate) fn answer(&self, request_line: &str) -> Answer {
        let (reply, keeps_serving) = match parse_daemon_request(request_line) {
            Ok(DaemonRequest::Status) => (Ok(daemon_status_result(&self.status)), true),
            Ok(DaemonRequest::Stop) => (Ok(json!({ "stopping": true })), false),
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
