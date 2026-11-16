use super::{Effect, Ending, Finder, ReadlineMeaning, Remark, RowMatcher, readline_meaning};
use crate::{
    Action, Edit, Key, TextField, ViewKey, ViewResponse, act, chooser_entries, groups, press,
    quickfix_entries,
};

pub const NO_FILE_LOCATIONS: &str = "No file locations in the selection.";

const COPIED: &str = "Copied";

impl<M: RowMatcher> Finder<M> {
    pub(super) fn press(&mut self, key: &str) -> Vec<Effect> {
        let cursor = self.cursor;
        let effects = self
            .engine_key(key)
            .or_else(|| self.row_key(key))
            .or_else(|| self.readline_key(key))
            .unwrap_or_else(|| {
                self.type_text(key);
                vec![Effect::Draw]
            });
        if self.cursor != cursor {
            self.remark = None;
            self.help_shown = false;
        }
        effects
    }

    fn engine_key(&mut self, key: &str) -> Option<Vec<Effect>> {
        match key {
            "esc" => self.respond(ViewKey::Back, None),
            "ctrl-space" => self.respond(ViewKey::Choose, None),
            "enter" => self.respond(ViewKey::Enter(self.cursor), Some(Key::enter())),
            "ctrl-c" => Some(vec![Effect::CloseWithoutFinding]),
            "alt-o" => Some(vec![Effect::Close(self.found(Ending::SendRows))]),
            "ctrl-q" => Some(self.save_quickfix()),
            "tab" => Some(self.drawn(|finder| finder.mark_and_move(1))),
            "shift-tab" => Some(self.drawn(|finder| finder.mark_and_move(-1))),
            "down" | "ctrl-n" => Some(self.drawn(|finder| finder.move_cursor(1))),
            "up" | "ctrl-p" => Some(self.drawn(|finder| finder.move_cursor(-1))),
            "page-down" => Some(self.drawn(|finder| finder.move_pages(1))),
            "page-up" => Some(self.drawn(|finder| finder.move_pages(-1))),
            "f1" => Some(self.drawn(|finder| finder.help_shown = true)),
            _ => None,
        }
    }

    fn row_key(&mut self, key: &str) -> Option<Vec<Effect>> {
        let key = Key::parse(key)?;
        self.respond(ViewKey::Other, Some(key))
    }

    fn readline_key(&mut self, key: &str) -> Option<Vec<Effect>> {
        let effect = match readline_meaning(key)? {
            ReadlineMeaning::Close => Effect::CloseWithoutFinding,
            ReadlineMeaning::DeleteForwardOrClose if self.query.is_empty() => {
                Effect::CloseWithoutFinding
            }
            ReadlineMeaning::DeleteForwardOrClose => return Some(self.edited(Edit::DeleteForward)),
            ReadlineMeaning::Edit(edit) => return Some(self.edited(edit)),
            ReadlineMeaning::Down => return Some(self.drawn(|finder| finder.move_cursor(1))),
            ReadlineMeaning::Up => return Some(self.drawn(|finder| finder.move_cursor(-1))),
            ReadlineMeaning::Redraw => Effect::Draw,
        };
        Some(vec![effect])
    }

    pub(super) fn respond(&mut self, view_key: ViewKey, key: Option<Key>) -> Option<Vec<Effect>> {
        let groups = groups(&self.rows);
        let (view, response) = press(&self.state.view, view_key, &self.state.sources, &groups);
        self.state.view = view;
        let effects = match response {
            ViewResponse::Accept => return self.act_on(&key?),
            ViewResponse::Stay => return None,
            ViewResponse::Back => vec![Effect::Close(self.found(Ending::Back))],
            ViewResponse::Switch(source) => {
                self.state.source = Some(source);
                vec![Effect::Close(self.found(Ending::SwitchSource))]
            }
            ViewResponse::OpenChooser => {
                let entries = chooser_entries(&self.state.sources, &groups);
                self.entries = entries.into_iter().map(str::to_string).collect();
                self.drawn(Self::restart_query)
            }
            ViewResponse::CloseChooser => self.drawn(Self::restart_query),
            ViewResponse::Redraw => self.drawn(Self::rank),
        };
        Some(effects)
    }

    fn act_on(&mut self, key: &Key) -> Option<Vec<Effect>> {
        let action = act(&self.chosen_rows(), key);
        match action {
            Ok(Action::Nothing) => None,
            Ok(Action::CopyStay(text)) => {
                self.remark = Some(Remark::Done(COPIED.to_string()));
                Some(vec![Effect::Copy(text), Effect::Draw])
            }
            Ok(_) => Some(vec![Effect::Close(self.found(Ending::Key(key.clone())))]),
            Err(refusal) => {
                self.remark = Some(Remark::Refused(refusal.to_string()));
                Some(vec![Effect::Draw])
            }
        }
    }

    fn save_quickfix(&mut self) -> Vec<Effect> {
        let entries = quickfix_entries(&self.chosen_rows());
        if entries.is_empty() {
            self.remark = Some(Remark::Refused(NO_FILE_LOCATIONS.to_string()));
            return vec![Effect::Draw];
        }
        let found = self.found(Ending::SaveQuickfix);
        vec![Effect::SaveQuickfix(entries), Effect::Close(found)]
    }

    fn edited(&mut self, edit: Edit) -> Vec<Effect> {
        self.drawn(|finder| finder.edit(edit))
    }

    fn drawn(&mut self, change: impl FnOnce(&mut Self)) -> Vec<Effect> {
        change(self);
        vec![Effect::Draw]
    }
}

#[cfg(test)]
mod tests;
