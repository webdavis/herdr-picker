use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Timers {
    jobs: BTreeMap<String, Job>,
}

#[derive(Debug)]
struct Job {
    every_seconds: u64,
    next_at: u64,
}

impl Job {
    fn advance_past(&mut self, now: u64) {
        let missed_rounds = (now - self.next_at) / self.every_seconds;
        self.next_at += (missed_rounds + 1) * self.every_seconds;
    }
}

impl Timers {
    pub fn every(&mut self, name: &str, seconds: u64, now: u64) {
        let every_seconds = seconds.max(1);
        let job = Job {
            every_seconds,
            next_at: now + every_seconds,
        };
        self.jobs.insert(name.to_string(), job);
    }

    pub fn cancel(&mut self, name: &str) {
        self.jobs.remove(name);
    }

    pub fn due(&mut self, now: u64) -> Vec<String> {
        self.jobs
            .iter_mut()
            .filter(|(_, job)| job.next_at <= now)
            .map(|(name, job)| {
                job.advance_past(now);
                name.clone()
            })
            .collect()
    }

    pub fn next_at(&self) -> Option<u64> {
        self.jobs.values().map(|job| job.next_at).min()
    }
}

#[cfg(test)]
mod tests;
