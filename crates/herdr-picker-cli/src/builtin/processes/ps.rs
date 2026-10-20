use std::collections::HashMap;
use std::path::Path;

use herdr_picker_domain::{Action, PickerRow, Preview};

use crate::builtin::page::fields;
use crate::builtin::paint::{dim, teal, yellow};
use crate::builtin::rows::key;

// `ps -axo pid=,ppid=,user=,comm=` beside `lsof -nP -i -FpcnT`: a row a process.
pub(super) fn process_rows(ps: &str, lsof: &str) -> Vec<PickerRow> {
    let held = connections(lsof);
    ps.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let (pid, parent, user) = (parts.next()?, parts.next()?, parts.next()?);
            pid.bytes()
                .all(|byte| byte.is_ascii_digit())
                .then_some(())?;
            // The program's path may hold spaces, so it is the rest of the line.
            let mut program = line.trim_start();
            for _ in 0..3 {
                program = program.split_once(char::is_whitespace)?.1.trim_start();
            }
            let program = program.trim_end();
            Some(process_row(pid, parent, user, program, held.get(pid)))
        })
        .collect()
}

fn process_row(
    pid: &str,
    parent: &str,
    user: &str,
    program: &str,
    ports: Option<&Vec<String>>,
) -> PickerRow {
    let path = Path::new(program);
    let name = path
        .file_name()
        .map_or(program.into(), |name| name.to_string_lossy());
    let folder = path.parent().unwrap_or(Path::new("")).to_string_lossy();
    let ports = ports.map(Vec::as_slice).unwrap_or_default();
    let mut label = format!("{}  {name}  {}", yellow(&format!("{pid:>7}")), dim(&folder));
    let mut preview = format!(
        "{name}\n\n{}",
        fields(&[
            ("PID", pid.to_string()),
            ("Parent PID", parent.to_string()),
            ("User", user.to_string()),
            ("Program", program.to_string()),
        ])
    );
    if !ports.is_empty() {
        label.push_str(&format!("  {}", teal(&ports.join("; "))));
        preview.push_str(&format!(
            "\n\nOpen connections and ports\n{}",
            ports.join("\n")
        ));
    }
    PickerRow {
        label,
        preview: Some(Preview::Text(preview)),
        enter: Action::Insert(pid.to_string()),
        keys: vec![(
            key("alt-d"),
            Action::Insert(format!("lsof -nP -a -p {pid} -i")),
        )],
        group: None,
    }
}

// lsof's field output: a `p` line starts each process, an `n` line names a socket, and a
// `TST=` line after it gives that socket's TCP state.
fn connections(lsof: &str) -> HashMap<&str, Vec<String>> {
    let mut held: HashMap<&str, Vec<String>> = HashMap::new();
    let mut pid = None;
    for line in lsof.lines() {
        if let Some(number) = line
            .strip_prefix('p')
            .filter(|rest| rest.parse::<u32>().is_ok())
        {
            pid = Some(number);
        } else if let (Some(pid), Some(name)) = (pid, line.strip_prefix('n')) {
            held.entry(pid).or_default().push(name.to_string());
        } else if let (Some(pid), Some(state)) = (pid, line.strip_prefix("TST="))
            && let Some(last) = held.get_mut(pid).and_then(|names| names.last_mut())
        {
            last.push(' ');
            last.push_str(state);
        }
    }
    held
}

#[cfg(test)]
mod tests;
