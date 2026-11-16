use pwe_domain::{Picker, Stroke, launcher_bases};

const KEY_FUNCTION: &str = r#"local terminal = "com.superlogical.terminal"
local function key(base)
  return function(ctx)
    local view, view_failed = rex.session.view{}
    local process, process_failed = rex.block.call(terminal, "process", { block_id = ctx.block_id })
    local size, size_failed = rex.block.call(terminal, "size", { block_id = ctx.block_id })
    if view_failed or process_failed or size_failed then return false end
    local facts = rex.json.encode({ block = ctx.block_id, view = view, process = process, size = size })
    local answer, exec_failed = rex.exec({ pwe, "rex", "key", base }, { stdin = facts, timeout = 2 })
    if exec_failed or not answer or not answer.ok or answer.stdout == "" then return false end
    local request, decode_failed = rex.json.decode(answer.stdout)
    if decode_failed then return false end
    local _, open_failed = rex.call("session.new_layer", request)
    return not open_failed
  end
end
"#;

pub fn rex_lua(pwe: &str, pickers: &[Picker]) -> String {
    let mut script = format!("local pwe = {}\n{KEY_FUNCTION}", lua_string(pwe));
    for base in launcher_bases(pickers) {
        script.push_str(&format!(
            "rex.bind(\"{}\", key(\"{}\"))\n",
            rex_key_name(base),
            base.name()
        ));
    }
    script
}

fn rex_key_name(stroke: Stroke) -> String {
    match stroke {
        Stroke::Ctrl(letter) => format!("ctrl+{letter}"),
        Stroke::Alt(letter) => format!("alt+{letter}"),
        Stroke::Plain(letter) => letter.to_string(),
    }
}

fn lua_string(text: &str) -> String {
    let mut quoted = String::from("\"");
    for character in text.chars() {
        match character {
            '"' | '\\' => quoted.extend(['\\', character]),
            control if control.is_ascii_control() => {
                quoted.push_str(&format!("\\{:03}", u32::from(control)));
            }
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests;
