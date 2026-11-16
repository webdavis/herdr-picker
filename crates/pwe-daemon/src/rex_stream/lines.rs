use std::io::{BufRead, BufReader, Read};
use std::process::ChildStdout;
use std::sync::mpsc::SyncSender;

use pwe_domain::Backoff;
use pwe_protocol::{RexEvent, RexLink, parse_rex_event};

use super::told;
use crate::arrival::{Arrival, RexNews};

const LONGEST_EVENT_BYTES: u64 = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Ended {
    RexWentAway,
    Stopping,
}

pub(super) fn pass_on_events(
    output: ChildStdout,
    arrivals: &SyncSender<Arrival>,
    backoff: &mut Backoff,
) -> Ended {
    let mut reader = BufReader::new(output);
    let mut line = Vec::new();
    let mut line_number = 0;
    let mut unreadable = 0_u64;
    loop {
        line.clear();
        match reader
            .by_ref()
            .take(LONGEST_EVENT_BYTES)
            .read_until(b'\n', &mut line)
        {
            Ok(0) | Err(_) => return Ended::RexWentAway,
            Ok(_) => {}
        }
        line_number += 1;
        if line_number == 1 {
            backoff.succeeded();
            if !told(arrivals, RexLink::Connected) {
                return Ended::Stopping;
            }
        }
        let event = match whole_line(&mut reader, &line) {
            Some(text) => parse_rex_event(line_number, text).ok(),
            None => None,
        };
        match event {
            None => {
                unreadable += 1;
                eprintln!(
                    "pwe: skipped line {line_number} of Rex's events, which is unreadable ({unreadable} so far)"
                );
            }
            Some(RexEvent::Other) => {}
            Some(event) => {
                if arrivals.send(Arrival::Rex(RexNews::Event(event))).is_err() {
                    return Ended::Stopping;
                }
            }
        }
    }
}

fn whole_line<'a>(reader: &mut BufReader<ChildStdout>, line: &'a [u8]) -> Option<&'a str> {
    if !line.ends_with(b"\n") {
        let _ = reader.skip_until(b'\n');
        return None;
    }
    std::str::from_utf8(line).ok()
}
