use pwe_adapters::{mark_inbox_row_read, notice_rows};
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
        let changed = |outcome: super::Outcome| !outcome.saved.is_empty();
        Ok(match request {
            NoticeRequest::Notify(notify) => self.notify(notify, now),
            NoticeRequest::Resolve { key } => {
                json!({ "resolved": changed(self.apply(NoticeInput::Resolve { key }, now)) })
            }
            NoticeRequest::Waiting => {
                let waiting = self.apply(NoticeInput::ListWaiting, now).waiting;
                json!({ "waiting": notice_rows(&waiting).unwrap_or_default() })
            }
            NoticeRequest::Hide => {
                self.apply(NoticeInput::Hide, now);
                json!({ "hidden": true })
            }
            NoticeRequest::Jump { id } => {
                json!({ "jumped": changed(self.apply(NoticeInput::Jump { id }, now)) })
            }
            NoticeRequest::Dismiss { id } => json!({ "dismissed": self.read(id, now) }),
            NoticeRequest::Read { id } => json!({ "read": self.read(id, now) }),
            NoticeRequest::Mute { source, until } => {
                self.apply(
                    NoticeInput::Mute {
                        source: source.clone(),
                        until,
                    },
                    now,
                );
                json!({ "muted": source, "until": until })
            }
            NoticeRequest::Unmute { source } => {
                self.apply(
                    NoticeInput::Unmute {
                        source: source.clone(),
                    },
                    now,
                );
                json!({ "unmuted": source })
            }
        })
    }

    fn read(&mut self, id: String, now: u64) -> bool {
        if !self
            .apply(NoticeInput::Read { id: id.clone() }, now)
            .saved
            .is_empty()
        {
            return true;
        }
        mark_inbox_row_read(&self.parts.inbox, &id, now).unwrap_or_else(|failure| {
            self.report(format!("could not mark the inbox row {id} read: {failure}"));
            false
        })
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
