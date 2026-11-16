use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{self, SyncSender};
use std::time::Duration;

use crate::arrival::{Arrival, Request};

const REQUEST_DEADLINE: Duration = Duration::from_secs(2);
const LONGEST_REQUEST_BYTES: u64 = 64 * 1024;

pub(crate) fn listen(listener: &UnixListener, requests: &SyncSender<Arrival>) {
    for mut stream in listener.incoming().flatten() {
        if let Ok(false) = exchange(&mut stream, requests) {
            return;
        }
    }
}

fn exchange(stream: &mut UnixStream, requests: &SyncSender<Arrival>) -> io::Result<bool> {
    stream.set_read_timeout(Some(REQUEST_DEADLINE))?;
    stream.set_write_timeout(Some(REQUEST_DEADLINE))?;
    let mut line = String::new();
    BufReader::new(Read::by_ref(stream).take(LONGEST_REQUEST_BYTES)).read_line(&mut line)?;
    let (answer_to, answered) = mpsc::channel();
    requests
        .send(Arrival::Request(Request { line, answer_to }))
        .map_err(|_| io::Error::other("the daemon stopped answering"))?;
    let answer = answered
        .recv()
        .map_err(|_| io::Error::other("the daemon stopped answering"))?;
    stream.write_all(answer.reply_line.as_bytes())?;
    Ok(answer.keeps_serving)
}
