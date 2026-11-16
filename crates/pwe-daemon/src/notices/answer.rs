use pwe_domain::{NoticeInput, Raised};
use pwe_protocol::{DaemonError, NoticeRequest, NotifyRequest};
use serde_json::{Value, json};

use super::NoticeDesk;

impl NoticeDesk {
    pub(crate) fn answer(
        &mut self,
        request: NoticeRequest,
        now: u64,
    ) -> Result<Value, DaemonError> {
        match request {
            NoticeRequest::Notify(notify) => Ok(self.notify(notify, now)),
            NoticeRequest::Resolve { key } => {
                let resolved = self.apply(NoticeInput::Resolve { key }, now);
                Ok(json!({ "resolved": !resolved.saved.is_empty() }))
            }
        }
    }

    fn notify(&mut self, request: NotifyRequest, now: u64) -> Value {
        let focus = self.owner_focus();
        self.apply(NoticeInput::OwnerMoved(focus), now);
        let placed = request.pane.as_deref().and_then(|block| self.placed(block));
        let raised = Raised {
            key: request.key,
            source: request.source,
            kind: request.kind,
            level: request.level,
            text: request.text,
            tab_label: placed.as_ref().map(|placed| placed.tab_label.clone()),
            pane: placed.map(|placed| placed.pane),
        };
        let raised = self.apply(NoticeInput::Raise(raised), now);
        let id = raised.saved.first().map(|row| row.id.clone());
        json!({ "id": id.unwrap_or_default() })
    }
}
