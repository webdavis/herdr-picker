#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRun {
    pub run_id: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    pub keep_runs: usize,
    pub keep_bytes: u64,
}

impl Default for Retention {
    fn default() -> Self {
        Retention {
            keep_runs: 100,
            keep_bytes: 256 * 1024 * 1024,
        }
    }
}

pub fn runs_to_drop(retention: &Retention, runs: &[StoredRun]) -> Vec<String> {
    let mut newest_first: Vec<&StoredRun> = runs.iter().collect();
    newest_first.sort_by(|left, right| right.run_id.cmp(&left.run_id));
    let kept = kept_count(retention, &newest_first);
    newest_first[kept..]
        .iter()
        .rev()
        .map(|run| run.run_id.clone())
        .collect()
}

fn kept_count(retention: &Retention, newest_first: &[&StoredRun]) -> usize {
    let mut bytes = 0;
    newest_first
        .iter()
        .take(retention.keep_runs)
        .enumerate()
        .take_while(|(index, run)| {
            bytes += run.bytes;
            *index == 0 || bytes <= retention.keep_bytes
        })
        .count()
}

#[cfg(test)]
mod tests;
