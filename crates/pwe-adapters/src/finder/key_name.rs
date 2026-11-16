use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn key_name(event: KeyEvent) -> Option<String> {
    if event.kind == KeyEventKind::Release {
        return None;
    }
    let control = event.modifiers.contains(KeyModifiers::CONTROL);
    let alt = event.modifiers.contains(KeyModifiers::ALT);
    let (key, shifted) = match event.code {
        KeyCode::Char(' ') => ("space".to_string(), false),
        KeyCode::Char(character) if control => (character.to_lowercase().collect(), false),
        KeyCode::Char(character) => (character.to_string(), false),
        KeyCode::BackTab => ("tab".to_string(), true),
        KeyCode::F(number) => (format!("f{number}"), shift_held(event)),
        code => (named_key(code)?.to_string(), shift_held(event)),
    };
    let prefixes = [(control, "ctrl-"), (alt, "alt-"), (shifted, "shift-")];
    let prefix: String = prefixes
        .into_iter()
        .filter(|(held, _)| *held)
        .map(|(_, prefix)| prefix)
        .collect();
    Some(prefix + &key)
}

fn shift_held(event: KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::SHIFT)
}

fn named_key(code: KeyCode) -> Option<&'static str> {
    Some(match code {
        KeyCode::Enter => "enter",
        KeyCode::Tab => "tab",
        KeyCode::Esc => "esc",
        KeyCode::Backspace => "bspace",
        KeyCode::Delete => "del",
        KeyCode::Insert => "insert",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "page-up",
        KeyCode::PageDown => "page-down",
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
