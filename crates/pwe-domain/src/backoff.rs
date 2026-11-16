const FIRST_WAIT_SECONDS: u64 = 1;
const LONGEST_WAIT_SECONDS: u64 = 30;

#[derive(Debug)]
pub struct Backoff {
    next_wait_seconds: u64,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            next_wait_seconds: FIRST_WAIT_SECONDS,
        }
    }
}

impl Backoff {
    pub fn wait_after_failure(&mut self) -> u64 {
        let wait = self.next_wait_seconds;
        self.next_wait_seconds = (wait * 2).min(LONGEST_WAIT_SECONDS);
        wait
    }

    pub fn succeeded(&mut self) {
        self.next_wait_seconds = FIRST_WAIT_SECONDS;
    }
}

#[cfg(test)]
mod tests;
