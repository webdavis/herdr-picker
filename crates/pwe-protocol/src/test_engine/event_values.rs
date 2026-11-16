use pwe_domain::test_engine::{Target, TestId, UntrustedText};
use serde_json::Value;

#[derive(Clone, Copy)]
pub(super) struct Object<'a>(pub(super) &'a Value);

impl<'a> Object<'a> {
    pub(super) fn word(self, key: &str) -> Option<&'a str> {
        self.0.get(key)?.as_str()
    }

    pub(super) fn text(self, key: &str) -> Option<String> {
        self.word(key).map(str::to_string)
    }

    pub(super) fn count(self, key: &str) -> Option<u64> {
        self.0.get(key)?.as_u64()
    }

    pub(super) fn number(self, key: &str) -> Option<u32> {
        u32::try_from(self.count(key)?).ok()
    }

    pub(super) fn id(self, key: &str) -> Option<TestId> {
        self.word(key)?.parse().ok()
    }

    pub(super) fn untrusted(self, key: &str) -> Option<UntrustedText> {
        self.word(key).map(UntrustedText::new)
    }

    pub(super) fn object(self, key: &str) -> Option<Object<'a>> {
        self.0
            .get(key)
            .filter(|value| value.is_object())
            .map(Object)
    }

    pub(super) fn objects(self, key: &str) -> Option<impl Iterator<Item = Object<'a>>> {
        Some(self.0.get(key)?.as_array()?.iter().map(Object))
    }

    pub(super) fn words(self, key: &str) -> Option<Vec<String>> {
        self.0
            .get(key)?
            .as_array()?
            .iter()
            .map(|word| word.as_str().map(str::to_string))
            .collect()
    }
}

pub(super) fn read_target(target: Object) -> Option<Target> {
    Some(match target.word("kind")? {
        "nearest" => {
            let (file, line) = target.word("value")?.rsplit_once(':')?;
            Target::Nearest {
                file: file.to_string(),
                line: line.parse().ok()?,
            }
        }
        "file" => Target::File(target.text("value")?),
        "ids" => Target::Ids(
            target
                .words("value")?
                .iter()
                .map(|id| id.parse().ok())
                .collect::<Option<_>>()?,
        ),
        "failed" => Target::Failed,
        "last" => Target::Last,
        "changed" => Target::Changed,
        "all" => Target::All,
        _ => return None,
    })
}
