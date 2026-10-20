// The pickers herdr-picker ships, on their keys unless the config turns them off or moves them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinPicker {
    pub name: &'static str,
    pub key: &'static str,
    pub description: &'static str,
    // The first is the one a picker opens on.
    pub sources: &'static [&'static str],
}

pub const BUILTIN_PICKERS: [BuiltinPicker; 3] = [
    BuiltinPicker {
        name: "files",
        key: "ctrl-t f",
        description: "Files here, hidden ones, at home, on the whole disk, or used lately",
        sources: &["this folder", "hidden", "home", "whole disk", "recent"],
    },
    BuiltinPicker {
        name: "folders",
        key: "ctrl-t d",
        description: "Folders to change into",
        sources: &["subfolders", "parents", "git", "bookmarks"],
    },
    BuiltinPicker {
        name: "bookmark",
        key: "ctrl-t ctrl-b",
        description: "Bookmark this folder for the folders picker",
        sources: &[],
    },
];
