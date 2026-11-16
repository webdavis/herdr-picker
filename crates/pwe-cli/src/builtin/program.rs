use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

// What a program printed by the time it ended, or by its deadline, when it is stopped.
pub(super) struct Ran {
    pub(super) stdout: String,
    pub(super) stderr: String,
    pub(super) succeeded: bool,
}

pub(super) fn ran(argv: &[&str], input: Option<&str>, timeout: Duration) -> Result<Ran, String> {
    let (program, arguments) = argv.split_first().ok_or("no program")?;
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|failure| format!("{program}: {failure}"))?;
    if let (Some(text), Some(mut stdin)) = (input, child.stdin.take()) {
        let _ = stdin.write_all(text.as_bytes());
    }
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let text = |reader: thread::JoinHandle<Vec<u8>>| {
        String::from_utf8_lossy(&reader.join().unwrap_or_default()).into_owned()
    };
    Ok(Ran {
        stdout: text(stdout),
        stderr: text(stderr),
        succeeded: status.is_some_and(|status| status.success()),
    })
}

// What a program printed when it succeeded in time; why not otherwise.
pub(super) fn run(argv: &[&str], timeout: Duration) -> Result<String, String> {
    let done = ran(argv, None, timeout)?;
    match done.succeeded {
        true => Ok(done.stdout),
        false => Err(format!("{} failed: {}", argv[0], done.stderr.trim())),
    }
}

fn drain(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut read = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut read);
        }
        read
    })
}

#[cfg(test)]
mod tests;
