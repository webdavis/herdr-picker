use super::{TestEvent, TestId, TestStatus};

pub fn failed_ids(events: &[TestEvent]) -> Vec<TestId> {
    let mut ids: Vec<TestId> = Vec::new();
    for event in events {
        if let TestEvent::TestFinished(finished) = event
            && matches!(finished.status, TestStatus::Failed | TestStatus::Errored)
            && !ids.contains(&finished.id)
        {
            ids.push(finished.id.clone());
        }
    }
    ids
}

#[cfg(test)]
mod tests;
