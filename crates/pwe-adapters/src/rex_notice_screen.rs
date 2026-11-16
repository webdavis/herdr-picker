use std::path::PathBuf;

use pwe_application::{JumpError, NoticeFloat, NoticeScreen};
use pwe_domain::{Corner, notice_place};
use pwe_protocol::{
    block_label_request, close_layer_request, focused_block_in_view, notice_float_request,
    opened_layer, session_request, window_in_view,
};

use crate::RexPanes;
use crate::response::jump_error;

struct ShownFloat {
    float: String,
    rex_session: String,
    layer: String,
    block: String,
}

pub struct RexNoticeScreen {
    rex: RexPanes,
    pwe: String,
    corner: Corner,
    shown: Vec<ShownFloat>,
}

impl RexNoticeScreen {
    pub fn at(rex: PathBuf, pwe: String, corner: Corner) -> Self {
        Self {
            rex: RexPanes::at(rex, None),
            pwe,
            corner,
            shown: Vec::new(),
        }
    }

    fn call(&self, method: &str, request: &str) -> Result<String, JumpError> {
        self.rex.run(&["api", "call", method, request])
    }

    fn shown(&self, float: &str) -> Option<&ShownFloat> {
        self.shown.iter().find(|shown| shown.float == float)
    }

    fn program(&self, notice: &NoticeFloat) -> Vec<String> {
        let until = notice.until.to_string();
        [
            &self.pwe,
            "notice",
            "--level",
            notice.level.word(),
            "--until",
            &until,
        ]
        .map(str::to_string)
        .to_vec()
    }
}

impl NoticeScreen for RexNoticeScreen {
    fn show(&mut self, notice: &NoticeFloat) -> Result<(), JumpError> {
        let session = notice.rex_session.as_str();
        let view = self.call("session.view", &session_request(session))?;
        let focused = focused_block_in_view(&view).map_err(jump_error)?;
        let size = self.rex.run(&["block", "call", "-b", &focused, "size"])?;
        let window = window_in_view(&view, &focused, &size).map_err(jump_error)?;
        let place = notice_place(window, self.corner, notice.slot, &notice.line);
        let request = notice_float_request(session, &place, &notice.line, &self.program(notice));
        let opened =
            opened_layer(&self.call("session.new_layer", &request)?).map_err(jump_error)?;
        self.shown.push(ShownFloat {
            float: notice.float.clone(),
            rex_session: session.to_string(),
            layer: opened.layer,
            block: opened.block,
        });
        Ok(())
    }

    fn relabel(&mut self, float: &str, line: &str) -> Result<(), JumpError> {
        let Some(shown) = self.shown(float) else {
            return Ok(());
        };
        let request = block_label_request(&shown.rex_session, &shown.block, line);
        self.call("session.set_block_label", &request).map(|_| ())
    }

    fn close(&mut self, float: &str) -> Result<(), JumpError> {
        let Some(index) = self.shown.iter().position(|shown| shown.float == float) else {
            return Ok(());
        };
        let shown = self.shown.remove(index);
        let request = close_layer_request(&shown.rex_session, &shown.layer);
        self.call("session.close_layer", &request).map(|_| ())
    }
}

#[cfg(test)]
mod tests;
