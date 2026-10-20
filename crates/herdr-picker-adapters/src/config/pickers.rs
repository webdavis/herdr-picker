use std::path::Path;

use herdr_picker_domain::PickerDeclaration;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PickerEntry {
    key: String,
    #[serde(default)]
    description: String,
    command: String,
}

impl PickerEntry {
    pub(super) fn declared(self, name: String, pickers_directory: &Path) -> PickerDeclaration {
        PickerDeclaration {
            name,
            key: self.key,
            description: self.description,
            command: pickers_directory.join(self.command),
        }
    }
}
