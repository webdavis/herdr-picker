use std::collections::HashMap;

use pwe_domain::Timers;
use pwe_protocol::{
    DaemonError, DaemonErrorKind, DaemonRequest, DaemonStatus, daemon_reply_line,
    daemon_status_result, parse_daemon_request, parse_notice_request,
};
use serde_json::{Value, json};

use crate::notices::NoticeDesk;

pub(crate) type Job = Box<dyn FnMut(u64) + Send>;

pub(crate) struct DaemonCore {
    status: DaemonStatus,
    timers: Timers,
    jobs: HashMap<String, Job>,
    notices: Option<NoticeDesk>,
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
            notices: None,
        }
    }

    pub(crate) fn every(&mut self, name: &str, seconds: u64, now: u64, job: Job) {
        self.timers.every(name, seconds, now);
        self.jobs.insert(name.to_string(), job);
    }

    pub(crate) fn attach_notices(&mut self, desk: NoticeDesk) {
        self.notices = Some(desk);
    }

    pub(crate) fn run_due(&mut self, now: u64) {
        if let Some(notices) = &mut self.notices {
            notices.run_due(now);
        }
        for name in self.timers.due(now) {
            if let Some(job) = self.jobs.get_mut(&name) {
                job(now);
            }
        }
    }

    pub(crate) fn next_at(&self) -> Option<u64> {
        let notices_at = self.notices.as_ref().and_then(NoticeDesk::next_at);
        self.timers.next_at().into_iter().chain(notices_at).min()
    }

    pub(crate) fn answer(&mut self, request_line: &str, now: u64) -> Answer {
        let (reply, keeps_serving) = match parse_daemon_request(request_line) {
            Ok(DaemonRequest::Status) => (Ok(daemon_status_result(&self.status)), true),
            Ok(DaemonRequest::Stop) => (Ok(json!({ "stopping": true })), false),
            Err(refusal) if refusal.kind == DaemonErrorKind::UnknownRequest => (
                self.answer_notices(request_line, now)
                    .unwrap_or(Err(refusal)),
                true,
            ),
            Err(refusal) => (Err(refusal), true),
        };
        Answer {
            reply_line: daemon_reply_line(&reply),
            keeps_serving,
        }
    }

    fn answer_notices(
        &mut self,
        request_line: &str,
        now: u64,
    ) -> Option<Result<Value, DaemonError>> {
        let request = parse_notice_request(request_line)?;
        let notices = self.notices.as_mut()?;
        Some(request.and_then(|request| notices.answer(request, now)))
    }
}

#[cfg(test)]
mod tests;
