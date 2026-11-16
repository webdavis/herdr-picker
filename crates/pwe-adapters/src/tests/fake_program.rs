use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FOLDER: AtomicU64 = AtomicU64::new(0);

pub(crate) struct FakePrograms {
    folder: PathBuf,
}

pub(crate) struct Answer<'a> {
    pub(crate) stdout: &'a str,
    pub(crate) stderr: &'a str,
    pub(crate) exit_code: u8,
}

fn quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

impl FakePrograms {
    pub(crate) fn new() -> Self {
        let number = NEXT_FOLDER.fetch_add(1, Ordering::Relaxed);
        let folder =
            std::env::temp_dir().join(format!("pwe-fake-programs-{}-{number}", std::process::id()));
        std::fs::create_dir(&folder).expect("create the fake programs folder");
        Self { folder }
    }

    pub(crate) fn answering(self, name: &str, answer: Answer) -> Self {
        let record = |suffix: &str| {
            quoted(
                &self
                    .folder
                    .join(format!("{name}.{suffix}"))
                    .to_string_lossy(),
            )
        };
        let script = format!(
            "#!/bin/sh\nprintf '%s\\0' \"$@\" > {}\n/bin/cat > {}\nprintf '%s' {}\nprintf '%s' {} >&2\nexit {}\n",
            record("arguments"),
            record("input"),
            quoted(answer.stdout),
            quoted(answer.stderr),
            answer.exit_code
        );
        let program = self.folder.join(name);
        std::fs::write(&program, script).expect("write the fake program");
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700))
            .expect("make the fake program executable");
        self
    }

    pub(crate) fn path_variable(&self) -> OsString {
        self.folder.clone().into_os_string()
    }

    pub(crate) fn arguments(&self, name: &str) -> Option<Vec<String>> {
        let bytes = std::fs::read(self.folder.join(format!("{name}.arguments"))).ok()?;
        Some(
            bytes
                .split(|byte| *byte == 0)
                .filter(|argument| !argument.is_empty())
                .map(|argument| String::from_utf8_lossy(argument).into_owned())
                .collect(),
        )
    }

    pub(crate) fn input(&self, name: &str) -> String {
        std::fs::read_to_string(self.folder.join(format!("{name}.input"))).unwrap_or_default()
    }
}

impl Drop for FakePrograms {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}
