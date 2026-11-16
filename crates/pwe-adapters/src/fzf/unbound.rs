use pwe_domain::Edit;
use pwe_domain::finder::{ReadlineMeaning, readline_meaning};

pub fn unbound_action(key: &str) -> Option<&'static str> {
    Some(match readline_meaning(key)? {
        ReadlineMeaning::Edit(edit) => fzf_edit(edit)?,
        ReadlineMeaning::DeleteForwardOrClose => "delete-char/eof",
        ReadlineMeaning::Up => "up",
        ReadlineMeaning::Down => "down",
        ReadlineMeaning::Close => "abort",
        ReadlineMeaning::Redraw => "clear-screen",
    })
}

fn fzf_edit(edit: Edit) -> Option<&'static str> {
    Some(match edit {
        Edit::Start => "beginning-of-line",
        Edit::End => "end-of-line",
        Edit::Backward => "backward-char",
        Edit::Forward => "forward-char",
        Edit::WordBackward => "backward-word",
        Edit::WordForward => "forward-word",
        Edit::DeleteBackward => "backward-delete-char",
        Edit::DeleteForward => "delete-char",
        Edit::CutToStart => "unix-line-discard",
        Edit::CutToEnd => "kill-line",
        Edit::CutWordBackward => "unix-word-rubout",
        Edit::CutWordForward => "kill-word",
        Edit::Paste => "yank",
        Edit::Insert(_) => return None,
    })
}
