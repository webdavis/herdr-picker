use std::io;

use pwe_domain::{Level, Notice, Pane};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RowPane {
    rex_session: String,
    tab: String,
    block: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct InboxRow {
    id: String,
    key: String,
    source: String,
    kind: String,
    level: String,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pane: Option<RowPane>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tab_label: Option<String>,
    raised_at: u64,
    updated_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolved_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    read_at: Option<u64>,
}

impl From<&Notice> for InboxRow {
    fn from(notice: &Notice) -> Self {
        Self {
            id: notice.id.clone(),
            key: notice.key.clone(),
            source: notice.source.clone(),
            kind: notice.kind.clone(),
            level: notice.level.word().to_string(),
            text: notice.text.clone(),
            pane: notice.pane.as_ref().map(|pane| RowPane {
                rex_session: pane.rex_session.clone(),
                tab: pane.tab.clone(),
                block: pane.block.clone(),
            }),
            tab_label: notice.tab_label.clone(),
            raised_at: notice.raised_at,
            updated_at: notice.updated_at,
            resolved_at: notice.resolved_at,
            read_at: notice.read_at,
        }
    }
}

impl TryFrom<InboxRow> for Notice {
    type Error = io::Error;

    fn try_from(row: InboxRow) -> io::Result<Self> {
        let level = Level::parse(&row.level).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown level {}", row.level),
            )
        })?;
        Ok(Self {
            id: row.id,
            key: row.key,
            source: row.source,
            kind: row.kind,
            level,
            text: row.text,
            pane: row.pane.map(|pane| Pane {
                rex_session: pane.rex_session,
                tab: pane.tab,
                block: pane.block,
            }),
            tab_label: row.tab_label,
            raised_at: row.raised_at,
            updated_at: row.updated_at,
            resolved_at: row.resolved_at,
            read_at: row.read_at,
        })
    }
}
