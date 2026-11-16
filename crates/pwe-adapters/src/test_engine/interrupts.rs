use std::io;
use std::thread::{self, JoinHandle};

use rustix::process::{Pid, Signal, kill_process_group};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::{Handle, Signals};

pub(super) struct InterruptsCaught {
    signals: Signals,
}

pub(super) struct InterruptsForwarded {
    handle: Handle,
    forwarder: JoinHandle<bool>,
}

impl InterruptsCaught {
    pub(super) fn catch() -> io::Result<Self> {
        Ok(InterruptsCaught {
            signals: Signals::new([SIGINT])?,
        })
    }

    pub(super) fn forward_to(self, group: Option<Pid>) -> InterruptsForwarded {
        let mut signals = self.signals;
        let handle = signals.handle();
        let forwarder = thread::spawn(move || {
            let mut interrupted = false;
            for _ in signals.forever() {
                interrupted = true;
                if let Some(group) = group {
                    let _ = kill_process_group(group, Signal::INT);
                }
            }
            interrupted
        });
        InterruptsForwarded { handle, forwarder }
    }
}

impl InterruptsForwarded {
    pub(super) fn interrupted(self) -> bool {
        self.handle.close();
        self.forwarder.join().unwrap_or(true)
    }
}
