use herdr_picker_domain::{Chord, PickerCommand};

use super::*;

fn picker(name: &str, key: &str) -> Picker {
    Picker {
        name: name.to_string(),
        chord: Chord::parse(key).expect("a chord"),
        description: String::new(),
        command: PickerCommand::Builtin("unused".to_string()),
        args: Vec::new(),
        title: String::new(),
        sources: Vec::new(),
        steps: Default::default(),
        live: false,
    }
}

#[test]
fn bash_init_defines_one_function_and_binds_each_picker_in_the_emacs_and_both_vi_keymaps() {
    let script = bash_init(&[picker("branches", "ctrl-g b"), picker("files", "alt-f")]);

    assert_eq!(
        script,
        r#"__herdr_picker() {
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
bind -m emacs-standard -x '"\C-gb": __herdr_picker branches'
bind -m vi-insert -x '"\C-gb": __herdr_picker branches'
bind -m vi-command -x '"\C-gb": __herdr_picker branches'
bind -m emacs-standard -x '"\ef": __herdr_picker files'
bind -m vi-insert -x '"\ef": __herdr_picker files'
bind -m vi-command -x '"\ef": __herdr_picker files'
"#
    );
}

#[test]
fn readline_keys_spell_ctrl_alt_and_plain_strokes() {
    let keys = |chord: &str| readline_keys(&Chord::parse(chord).expect("a chord"));

    assert_eq!(keys("ctrl-t b"), r"\C-tb");
    assert_eq!(keys("alt-g ctrl-x B"), r"\eg\C-xB");
}
