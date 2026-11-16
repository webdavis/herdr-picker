use pwe_domain::test_engine::{Failure, UntrustedText};

const PANICKED_AT: &str = " panicked at ";
const LEFT: &str = "  left: ";
const RIGHT: &str = " right: ";
const BACKTRACE_NOTES: [&str; 2] = ["note: run with `RUST_BACKTRACE", "stack backtrace:"];

pub(super) struct FailureBlock {
    pub(super) output: Option<UntrustedText>,
    pub(super) failure: Failure,
}

pub(super) fn read_failure_block(
    lines: &[String],
    own_place: Option<(String, u32)>,
) -> FailureBlock {
    let panic_at = lines.iter().position(|line| is_panic_line(line));
    let (printed, after_panic) = match panic_at {
        Some(index) => (&lines[..index], &lines[index + 1..]),
        None => (&lines[..0], lines),
    };
    let message: Vec<&str> = after_panic
        .iter()
        .map(String::as_str)
        .take_while(|line| !BACKTRACE_NOTES.iter().any(|note| line.starts_with(note)))
        .collect();
    let place = panic_at
        .and_then(|index| panic_place(&lines[index]))
        .filter(|(file, _)| !file.starts_with('/'))
        .or(own_place);
    FailureBlock {
        output: Some(joined(printed.iter().map(String::as_str)))
            .filter(|text| !text.is_empty())
            .map(|text| UntrustedText::new(&format!("{text}\n"))),
        failure: Failure {
            message: UntrustedText::new(&joined(
                message
                    .iter()
                    .copied()
                    .take_while(|line| !line.starts_with(LEFT)),
            )),
            file: place.as_ref().map(|(file, _)| file.clone()),
            line: place.map(|(_, line)| line),
            expected: value_after(&message, RIGHT),
            actual: value_after(&message, LEFT),
        },
    }
}

fn is_panic_line(line: &str) -> bool {
    line.starts_with("thread '") && line.contains(PANICKED_AT)
}

fn panic_place(line: &str) -> Option<(String, u32)> {
    let (_, place) = line.split_once(PANICKED_AT)?;
    let mut parts = place.strip_suffix(':')?.rsplitn(3, ':');
    let _column = parts.next()?;
    let line = parts.next()?.parse().ok()?;
    Some((parts.next()?.to_string(), line))
}

fn value_after(message: &[&str], label: &str) -> Option<UntrustedText> {
    message
        .iter()
        .find_map(|line| line.strip_prefix(label))
        .map(UntrustedText::new)
}

fn joined<'a>(lines: impl Iterator<Item = &'a str>) -> String {
    lines
        .collect::<Vec<_>>()
        .join("\n")
        .trim_matches('\n')
        .to_string()
}
