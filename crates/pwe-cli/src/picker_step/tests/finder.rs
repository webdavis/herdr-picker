use std::convert::Infallible;
use std::path::Path;

use pwe_application::{StepFinder, StepOutcome, StepSetup};
use pwe_domain::finder::{Ending, Found};
use pwe_domain::{Action, Key, PickerCall, PickerRow, Spot, Trail, View, ViewState};

use super::config;
use crate::picker_step::Run;

struct FoundFirstRow;

impl StepFinder for FoundFirstRow {
    type Error = Infallible;

    fn find(&self, _: StepSetup) -> Result<Option<StepOutcome>, Infallible> {
        let row = PickerRow {
            label: "alpha".to_string(),
            preview: None,
            enter: Action::Insert("a".to_string()),
            keys: Vec::new(),
            group: None,
        };
        Ok(Some(StepOutcome {
            found: Found {
                query: "al".to_string(),
                ending: Ending::Key(Key::enter()),
                rows: vec![row],
            },
            view_left: ViewState {
                view: View::List {
                    group: Some("docs".to_string()),
                },
                titles: Vec::new(),
                description: String::new(),
                source: Some("hidden".to_string()),
                sources: Vec::new(),
            },
        }))
    }
}

#[test]
fn show_turns_what_the_finder_found_into_an_action() {
    let config = config(
        "finder",
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = [\"/bin/ls\"]\n",
    );
    let run = Run {
        config: &config,
        finder: &FoundFirstRow,
        quickfix: Path::new("/q.json"),
    };
    let trail = Trail::new(PickerCall {
        name: "files".to_string(),
        args: Vec::new(),
        state: None,
        query: None,
    });

    let shown = run.show(&trail).expect("a step");

    assert_eq!(
        shown,
        Some((
            Action::Insert("a".to_string()),
            Spot {
                query: "al".to_string(),
                group: Some("docs".to_string()),
                source: Some("hidden".to_string()),
            }
        ))
    );
}
