use pwe_domain::test_engine::TestEvent;
use serde_json::{Map, Value};

use super::event_fields::event_fields;

const FORMAT_VERSION: u64 = 1;

pub struct EventLines {
    run_id: String,
    sequence: u64,
}

impl EventLines {
    pub fn new(run_id: &str) -> Self {
        EventLines {
            run_id: run_id.to_string(),
            sequence: 0,
        }
    }

    pub fn line(&mut self, time: &str, event: &TestEvent) -> String {
        self.sequence += 1;
        let (name, fields) = event_fields(event);
        let mut object = Map::new();
        object.insert("event".to_string(), name.into());
        if self.sequence == 1 {
            object.insert("v".to_string(), FORMAT_VERSION.into());
        }
        object.insert("run-id".to_string(), self.run_id.clone().into());
        object.insert("sequence".to_string(), self.sequence.into());
        object.insert("time".to_string(), time.into());
        object.extend(fields);
        Value::Object(object).to_string()
    }
}

#[cfg(test)]
mod tests;
