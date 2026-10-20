use herdr_picker_domain::WorkspaceDeclaration;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(
    untagged,
    expecting = "a directory, or a table of a dir and an optional key"
)]
pub(super) enum Declaration {
    Directory(String),
    Detailed(DetailedDeclaration),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DetailedDeclaration {
    dir: String,
    key: Option<String>,
}

impl Declaration {
    pub(super) fn labelled(self, label: String) -> WorkspaceDeclaration {
        match self {
            Self::Directory(directory) => WorkspaceDeclaration {
                label,
                directory,
                key: None,
            },
            Self::Detailed(detailed) => WorkspaceDeclaration {
                label,
                directory: detailed.dir,
                key: detailed.key,
            },
        }
    }
}
