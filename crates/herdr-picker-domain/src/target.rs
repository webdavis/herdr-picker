use crate::keymap::SEARCH_CHARACTER;

mod error;

pub use error::TargetError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDeclaration {
    pub label: String,
    pub directory: String,
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTarget {
    pub action_id: String,
    pub label: String,
    pub directory: String,
    pub pick_key: char,
}

pub fn action_id(label: &str) -> String {
    let mut id = String::from("jump_");
    for character in label.chars() {
        let lowered = character.to_ascii_lowercase();
        id.push(match lowered {
            'a'..='z' | '0'..='9' => lowered,
            _ => '_',
        });
    }
    id
}

fn pick_key(declaration: &WorkspaceDeclaration) -> Result<char, TargetError> {
    let Some(declared) = &declaration.key else {
        return declaration
            .label
            .chars()
            .next()
            .map(|first| first.to_ascii_lowercase())
            .filter(usable_as_pick_key)
            .ok_or_else(|| TargetError::LabelHasNoDefaultPickKey {
                label: declaration.label.clone(),
            });
    };
    let mut characters = declared.chars();
    characters
        .next()
        .filter(|_| characters.next().is_none())
        .filter(usable_as_pick_key)
        .ok_or_else(|| TargetError::UnusablePickKey {
            label: declaration.label.clone(),
            key: declared.clone(),
        })
}

fn usable_as_pick_key(character: &char) -> bool {
    character.is_ascii_graphic() && *character != SEARCH_CHARACTER
}

pub fn jump_targets<Declarations>(
    declarations: Declarations,
) -> Result<Vec<JumpTarget>, TargetError>
where
    Declarations: IntoIterator<Item = WorkspaceDeclaration>,
{
    let mut targets: Vec<JumpTarget> = Vec::new();
    for declaration in declarations {
        if declaration.label.is_empty() {
            return Err(TargetError::EmptyLabel);
        }
        if declaration.directory.is_empty() {
            return Err(TargetError::EmptyDirectory {
                label: declaration.label,
            });
        }
        let action_id = action_id(&declaration.label);
        if let Some(existing) = targets.iter().find(|target| target.action_id == action_id) {
            return Err(TargetError::CollidingActionId {
                first: existing.label.clone(),
                second: declaration.label,
                action_id,
            });
        }
        let pick_key = pick_key(&declaration)?;
        if let Some(existing) = targets.iter().find(|target| target.pick_key == pick_key) {
            return Err(TargetError::CollidingPickKey {
                first: existing.label.clone(),
                second: declaration.label,
                key: pick_key,
            });
        }
        targets.push(JumpTarget {
            action_id,
            label: declaration.label,
            directory: declaration.directory,
            pick_key,
        });
    }
    if targets.is_empty() {
        return Err(TargetError::NoWorkspaces);
    }
    Ok(targets)
}

#[cfg(test)]
mod tests;
