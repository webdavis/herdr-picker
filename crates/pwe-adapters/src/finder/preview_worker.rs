use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use pwe_domain::Preview;
use pwe_domain::finder::Input;

use crate::command_preview::command_preview;
use crate::path_preview::path_preview;

pub struct PreviewWorker {
    requests: Sender<Request>,
}

struct Request {
    row: usize,
    preview: Preview,
    width: usize,
}

impl PreviewWorker {
    pub fn start(answers: Sender<Input>, path_variable: Option<OsString>) -> Self {
        let (requests, asked) = mpsc::channel();
        thread::spawn(move || answer_the_newest(&asked, &answers, path_variable.as_deref()));
        Self { requests }
    }

    pub fn ask(&self, row: usize, preview: Preview, width: usize) {
        let _ = self.requests.send(Request {
            row,
            preview,
            width,
        });
    }
}

fn answer_the_newest(asked: &Receiver<Request>, answers: &Sender<Input>, path: Option<&OsStr>) {
    while let Ok(first) = asked.recv() {
        let newest = asked.try_iter().last().unwrap_or(first);
        let text = match newest.preview {
            Preview::Text(text) => text,
            Preview::Path(file) => path_preview(Path::new(&file), path, Some(newest.width)),
            Preview::Command(argv) => command_preview(&argv),
        };
        let answer = Input::PreviewArrived {
            row: newest.row,
            text,
        };
        if answers.send(answer).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests;
