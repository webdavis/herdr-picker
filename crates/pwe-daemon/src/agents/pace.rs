pub(crate) struct ReadingPace {
    refresh_seconds: u64,
    last_read_at: Option<u64>,
}

impl ReadingPace {
    pub(crate) fn every(refresh_seconds: u64) -> Self {
        Self {
            refresh_seconds,
            last_read_at: None,
        }
    }

    pub(crate) fn wait_before_reading(&self, now: u64) -> u64 {
        self.last_read_at
            .map_or(0, |last| (last + self.refresh_seconds).saturating_sub(now))
    }

    pub(crate) fn read_at(&mut self, now: u64) {
        self.last_read_at = Some(now);
    }
}

#[cfg(test)]
mod tests;
