use herdr_picker_domain::{Action, HerdrCall, PickerCall, PickerRow, Preview, unstyled};
use serde::{Deserialize, Serialize};

use super::agent_row::{status_glyph, tab_label, workspace_label};
use super::herdr_view::herdr_view;
use super::start::BROCCOLI;
use crate::builtin::page::notice;
use crate::builtin::paint::dim;
use crate::builtin::{Asked, Places};

mod answer;

pub(in crate::builtin) use answer::answer;
#[cfg(test)]
use answer::{answer_page, quick_answer_argv};

// What Alt-O carries from step to step: the marked rows as text, and what was asked of them.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(super) struct Sending {
    pub(super) text: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) from: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) question: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) account: String,
}

pub(crate) fn send_call(rows: &[PickerRow], from: &str) -> PickerCall {
    let lines: Vec<String> = rows.iter().map(|row| unstyled(&row.label)).collect();
    let sending = Sending {
        text: lines.join("\n"),
        from: from.to_string(),
        ..Sending::default()
    };
    step("send-to", &sending)
}

fn step(name: &str, sending: &Sending) -> PickerCall {
    PickerCall {
        name: name.to_string(),
        args: Vec::new(),
        state: serde_json::to_string(sending).ok(),
        query: None,
    }
}

pub(super) fn sending(asked: &Asked) -> Sending {
    asked
        .state
        .and_then(|state| serde_json::from_str(state).ok())
        .unwrap_or_default()
}

// Where the rows can go: a running agent, the clipboard, or a quick question to Claude.
pub(in crate::builtin) fn send_to(asked: &Asked, places: &Places) -> Vec<PickerRow> {
    let sending = sending(asked);
    let preview = || Some(Preview::Text(sending.text.clone()));
    let mut rows: Vec<PickerRow> = match herdr_view(&places.herdr, false) {
        Some(Ok(view)) => (view.snapshot.agents.iter())
            .map(|agent| PickerRow {
                label: format!(
                    "{} {}  {}",
                    status_glyph(agent.status),
                    agent.title,
                    dim(&format!(
                        "{} · {} › {}",
                        agent.kind,
                        workspace_label(&view, &agent.workspace_id),
                        tab_label(&view, &agent.tab_id)
                    ))
                ),
                preview: preview(),
                enter: Action::Herdr(HerdrCall::Prompt {
                    pane_id: agent.pane_id.clone(),
                    text: sending.text.clone(),
                }),
                keys: Vec::new(),
                group: None,
            })
            .collect(),
        _ => Vec::new(),
    };
    let row = |label: &str, enter| PickerRow {
        label: label.to_string(),
        preview: preview(),
        enter,
        keys: Vec::new(),
        group: None,
    };
    rows.push(row("copy instead", Action::Copy(sending.text.clone())));
    rows.push(row(
        "quick answer",
        Action::Next(step("quick-answer", &sending)),
    ));
    rows
}

// What is typed is the question; customer data goes only to the Broccoli account, so a Broccoli
// picker offers no other.
pub(in crate::builtin) fn quick_answer(asked: &Asked) -> Vec<PickerRow> {
    let sending = sending(asked);
    let question = asked.query.trim();
    if question.is_empty() {
        return vec![notice("Type your question", &sending.text)];
    }
    let accounts: &[&str] = match sending.from.to_lowercase().contains("broccoli") {
        true => &[BROCCOLI],
        false => &["personal", BROCCOLI],
    };
    accounts
        .iter()
        .map(|account| PickerRow {
            label: format!("Ask {account}: {question}"),
            preview: Some(Preview::Text(format!(
                "claude -p on the {account} account gets this question and these rows:\n\n{}",
                sending.text
            ))),
            enter: Action::Next(step(
                "answer",
                &Sending {
                    question: question.to_string(),
                    account: account.to_string(),
                    text: sending.text.clone(),
                    from: sending.from.clone(),
                },
            )),
            keys: Vec::new(),
            group: None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
