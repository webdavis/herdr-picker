use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use pwe_application::test_engine::Clock;

use crate::builtin::clock::compact_utc_time;

pub(super) struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| {
                u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
            })
    }
}

pub(super) fn new_run_id(now_ms: u64) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    let salt = (process::id() ^ nanos) & 0xffff;
    format!("{}-{salt:04x}", compact_utc_time(now_ms))
}
