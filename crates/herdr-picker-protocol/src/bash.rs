use herdr_picker_domain::{Chord, Picker, Stroke};

const APPLY_INSTRUCTION: &str = r#"__herdr_picker() {
  local instruction verb
  instruction=$(herdr-picker run "$1") || return
  verb=${instruction%%$'\t'*}
  instruction=${instruction#*$'\t'}
  case $verb in
  insert)
    READLINE_LINE=${READLINE_LINE:0:READLINE_POINT}$instruction${READLINE_LINE:READLINE_POINT}
    READLINE_POINT=$((READLINE_POINT + ${#instruction}))
    ;;
  cd) CDPATH='' builtin cd -- "$instruction" ;;
  run)
    history -s -- "$instruction"
    eval -- "$instruction"
    ;;
  esac
}
"#;

pub fn bash_init(pickers: &[Picker]) -> String {
    let mut script = APPLY_INSTRUCTION.to_string();
    for picker in pickers {
        script.push_str(&format!(
            "bind -x '\"{}\": __herdr_picker {}'\n",
            readline_keys(&picker.chord),
            picker.name
        ));
    }
    script
}

fn readline_keys(chord: &Chord) -> String {
    chord
        .strokes()
        .iter()
        .map(|stroke| match stroke {
            Stroke::Ctrl(letter) => format!("\\C-{letter}"),
            Stroke::Alt(letter) => format!("\\e{letter}"),
            Stroke::Plain(letter) => letter.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests;
