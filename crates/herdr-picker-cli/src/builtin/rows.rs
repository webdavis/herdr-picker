use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use herdr_picker_domain::{Action, Key, Location, PickerRow, Preview};
use herdr_picker_protocol::{encode_header, encode_row, shell_word};

use super::Places;

const BLUE: &str = "\x1b[38;2;137;180;250m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const FLUSH_EVERY: Duration = Duration::from_millis(16);

// The first row goes out at once and later ones in batches, so the list fills fast and cheaply.
pub(super) struct RowWriter<W: Write> {
    out: BufWriter<W>,
    flushed: Option<Instant>,
}

impl<W: Write> RowWriter<W> {
    pub(super) fn new(out: W) -> Self {
        Self {
            out: BufWriter::new(out),
            flushed: None,
        }
    }

    pub(super) fn row(&mut self, row: &PickerRow) -> io::Result<()> {
        let encoded = match row.label.contains(['\n', '\r', '\t']) {
            true => encode_row(&PickerRow {
                label: one_line(&row.label),
                ..row.clone()
            }),
            false => encode_row(row),
        };
        writeln!(self.out, "{encoded}")?;
        if self.flushed.is_none_or(|at| at.elapsed() >= FLUSH_EVERY) {
            self.out.flush()?;
            self.flushed = Some(Instant::now());
        }
        Ok(())
    }

    pub(super) fn header(&mut self, text: &str) -> io::Result<()> {
        writeln!(self.out, "{}", encode_header(text))
    }

    pub(super) fn finish(mut self) -> io::Result<()> {
        self.out.flush()
    }
}

// A label is one line: a log message's line breaks show the way the fzf-pickers suite drew them.
fn one_line(label: &str) -> String {
    label
        .replace('\r', "")
        .replace('\n', " ↵ ")
        .replace('\t', "  ")
}

pub(super) fn shown(path: &Path, cwd: &Path) -> String {
    match path.strip_prefix(cwd) {
        Ok(inside) if inside.as_os_str().is_empty() => ".".to_string(),
        Ok(inside) => inside.to_string_lossy().into_owned(),
        Err(_) => path.to_string_lossy().into_owned(),
    }
}

pub(super) fn in_home(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(inside) if inside.as_os_str().is_empty() => "~".to_string(),
        Some(inside) => format!("~/{}", inside.display()),
        None => path.display().to_string(),
    }
}

// Drawn the way the fzf-pickers suite draws a path.
pub(super) fn painted(shown: &str) -> String {
    match shown.rsplit_once('/') {
        Some((folder, name)) => format!("{BLUE}{folder}/{RESET}{name}"),
        None => shown.to_string(),
    }
}

// Drawn the way the fzf-pickers recent list draws a file.
pub(super) fn named(path: &Path, home: Option<&Path>) -> String {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let folder = in_home(path.parent().unwrap_or(path), home);
    format!("{name}  {DIM}{folder}{RESET}")
}

pub(super) fn file_row(
    path: &Path,
    label: String,
    places: &Places,
    toggle: Option<&str>,
) -> PickerRow {
    let full = path.to_string_lossy().into_owned();
    let shown = shown(path, &places.cwd);
    let folder = path.parent().unwrap_or(path).to_string_lossy().into_owned();
    let edit = Action::Edit(vec![Location {
        path: full.clone(),
        line: None,
    }]);
    let mut keys = vec![
        (key("ctrl-enter"), edit),
        (key("ctrl-o"), Action::Open(full.clone())),
        (key("alt-c"), Action::Cd(folder)),
        (key("alt-y"), Action::CopyStay(shown.clone())),
    ];
    keys.extend(toggle.map(|source| (key("alt-."), Action::Source(source.to_string()))));
    PickerRow {
        label,
        preview: Some(Preview::Path(full)),
        enter: Action::Insert(shell_word(&shown)),
        keys,
        group: None,
    }
}

pub(super) fn folder_row(path: &Path, places: &Places) -> PickerRow {
    let full = path.to_string_lossy().into_owned();
    let shown = shown(path, &places.cwd);
    PickerRow {
        label: format!("{BLUE}{shown}{RESET}"),
        preview: Some(Preview::Path(full.clone())),
        enter: Action::Cd(full.clone()),
        keys: vec![
            (key("alt-y"), Action::CopyStay(shown)),
            (key("ctrl-o"), Action::Open(full)),
        ],
        group: None,
    }
}

pub(super) fn key(name: &str) -> Key {
    Key::parse(name).expect("the built-in pickers claim only keys a row may claim")
}
