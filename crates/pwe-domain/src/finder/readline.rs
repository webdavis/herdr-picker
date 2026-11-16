use crate::Edit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadlineMeaning {
    Edit(Edit),
    DeleteForwardOrClose,
    Up,
    Down,
    Close,
    Redraw,
}

const FZF_MEANINGS_OF_UNCLAIMED_KEYS: [(&str, ReadlineMeaning); 25] = [
    ("ctrl-a", ReadlineMeaning::Edit(Edit::Start)),
    ("home", ReadlineMeaning::Edit(Edit::Start)),
    ("ctrl-b", ReadlineMeaning::Edit(Edit::Backward)),
    ("left", ReadlineMeaning::Edit(Edit::Backward)),
    ("ctrl-d", ReadlineMeaning::DeleteForwardOrClose),
    ("del", ReadlineMeaning::Edit(Edit::DeleteForward)),
    ("ctrl-e", ReadlineMeaning::Edit(Edit::End)),
    ("end", ReadlineMeaning::Edit(Edit::End)),
    ("ctrl-f", ReadlineMeaning::Edit(Edit::Forward)),
    ("right", ReadlineMeaning::Edit(Edit::Forward)),
    ("ctrl-g", ReadlineMeaning::Close),
    ("ctrl-h", ReadlineMeaning::Edit(Edit::DeleteBackward)),
    ("bspace", ReadlineMeaning::Edit(Edit::DeleteBackward)),
    ("ctrl-j", ReadlineMeaning::Down),
    ("ctrl-k", ReadlineMeaning::Up),
    ("ctrl-l", ReadlineMeaning::Redraw),
    ("ctrl-u", ReadlineMeaning::Edit(Edit::CutToStart)),
    ("ctrl-w", ReadlineMeaning::Edit(Edit::CutWordBackward)),
    ("alt-bspace", ReadlineMeaning::Edit(Edit::CutWordBackward)),
    ("ctrl-y", ReadlineMeaning::Edit(Edit::Paste)),
    ("alt-b", ReadlineMeaning::Edit(Edit::WordBackward)),
    ("shift-left", ReadlineMeaning::Edit(Edit::WordBackward)),
    ("alt-d", ReadlineMeaning::Edit(Edit::CutWordForward)),
    ("alt-f", ReadlineMeaning::Edit(Edit::WordForward)),
    ("shift-right", ReadlineMeaning::Edit(Edit::WordForward)),
];

pub fn readline_meaning(key: &str) -> Option<ReadlineMeaning> {
    FZF_MEANINGS_OF_UNCLAIMED_KEYS
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, meaning)| *meaning)
}
