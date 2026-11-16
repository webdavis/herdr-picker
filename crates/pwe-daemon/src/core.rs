use pwe_protocol::{
    DaemonRequest, DaemonStatus, daemon_reply_line, daemon_status_result, parse_daemon_request,
};
use serde_json::json;

pub(crate) struct DaemonCore {
    status: DaemonStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Answer {
    pub(crate) reply_line: String,
    pub(crate) keeps_serving: bool,
}

impl DaemonCore {
    pub(crate) fn new(status: DaemonStatus) -> Self {
        Self { status }
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
