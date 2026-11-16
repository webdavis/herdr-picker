use pwe_domain::test_engine::UntrustedText;
use serde_json::{Map, Value};

#[derive(Default)]
pub(super) struct Fields(Map<String, Value>);

impl Fields {
    pub(super) fn put(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.0.insert(key.to_string(), value.into());
        self
    }

    pub(super) fn put_some(self, key: &str, value: Option<impl Into<Value>>) -> Self {
        match value {
            Some(value) => self.put(key, value),
            None => self,
        }
    }

    pub(super) fn put_untrusted(self, texts: &[(&str, Option<&UntrustedText>)]) -> Self {
        let stripped: usize = texts
            .iter()
            .filter_map(|(_, text)| text.map(|text| text.stripped))
            .sum();
        texts
            .iter()
            .fold(self, |fields, (key, text)| {
                fields.put_some(key, text.map(|text| text.text.clone()))
            })
            .put("untrusted", true)
            .put("stripped", stripped)
    }

    pub(super) fn into_map(self) -> Map<String, Value> {
        self.0
    }
}

impl From<Fields> for Value {
    fn from(fields: Fields) -> Self {
        Value::Object(fields.0)
    }
}
