use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::thread;

use herdr_picker_domain::Action;

use super::steps::{ask, cancel, choice};
use super::{plural, wt};
use crate::builtin::rows::RowWriter;
use crate::command::CommandError;

mod repos;

pub(super) use repos::repos;

// Alt-X's step: what `wt step prune` would remove in each repository, then whether to.
pub(super) fn prune(roots: &[&str], rows: &mut RowWriter<impl Write>) -> Result<(), CommandError> {
    let listed = thread::scope(|scope| {
        let asking: Vec<_> = roots
            .iter()
            .map(|root| scope.spawn(move || dry_run(root, &[])))
            .collect();
        asking
            .into_iter()
            .map(|answer| answer.join().unwrap_or_default())
            .collect::<Vec<_>>()
    });
    let preview = roots
        .iter()
        .zip(listed)
        .map(|(root, said)| format!("{root}\n{}", said.trim_end()))
        .collect::<Vec<_>>()
        .join("\n\n");
    let question = match roots {
        [root] => format!("Prune {root}?"),
        _ => format!("Prune {}?", plural(roots.len(), "repo")),
    };
    let run = roots
        .iter()
        .map(|root| wt(Path::new(root), &["step", "prune"]))
        .collect::<Vec<_>>()
        .join("; ");
    let choices = vec![
        choice("Prune", "removes what the list shows", Action::Run(run)),
        cancel(),
    ];
    ask(&question, &preview, choices, rows)
}

pub(super) fn dry_run(root: &str, format: &[&str]) -> String {
    let mut command = Command::new("wt");
    command
        .args(["-C", root, "step", "prune", "--dry-run"])
        .args(format);
    match command.output() {
        Ok(ran) => {
            String::from_utf8_lossy(&ran.stdout).into_owned()
                + &String::from_utf8_lossy(&ran.stderr)
        }
        Err(failure) => format!("Cannot run wt: {failure}"),
    }
}
