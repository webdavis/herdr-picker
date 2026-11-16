use std::collections::{HashSet, VecDeque};

use serde_json::Value;

// A paged list read lazily, each record once; a failure is the last item.
pub(in crate::builtin::apple) fn pages(
    get: impl FnMut(usize) -> Result<Value, String>,
) -> impl Iterator<Item = Result<Value, String>> {
    let (mut get, mut offset, mut seen, mut ready, mut done) =
        (get, 0, HashSet::new(), VecDeque::new(), false);
    std::iter::from_fn(move || {
        loop {
            if let Some(item) = ready.pop_front() {
                return Some(Ok(item));
            }
            if done {
                return None;
            }
            let page = get(offset);
            let read = page.and_then(|page| {
                let batch = page["data"].as_array().cloned().unwrap_or_default();
                let mut fresh = 0;
                for item in &batch {
                    let guid = item["guid"].as_str().filter(|guid| !guid.is_empty());
                    let guid =
                        guid.ok_or("BlueBubbles returned a record without a stable identifier.")?;
                    if seen.insert(guid.to_string()) {
                        fresh += 1;
                        ready.push_back(item.clone());
                    }
                }
                offset += batch.len();
                let total = page["metadata"]["total"]
                    .as_u64()
                    .map(|total| total as usize);
                done = batch.is_empty() || total.is_some_and(|total| offset >= total);
                match fresh == 0 && !batch.is_empty() {
                    true => Err(
                        "BlueBubbles pagination did not advance; results are incomplete."
                            .to_string(),
                    ),
                    false => Ok(()),
                }
            });
            if let Err(why) = read {
                done = true;
                ready.clear();
                return Some(Err(why));
            }
        }
    })
}

#[cfg(test)]
mod tests;
