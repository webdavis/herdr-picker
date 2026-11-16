use pwe_domain::{Chord, PickerCommand};

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

fn lua(pickers: &[Picker]) -> String {
    rex_lua("/opt/pwe/bin/pwe", pickers)
}

#[test]
fn each_launcher_base_is_bound_once() {
    let script = lua(&[
        picker("files", "ctrl-t f"),
        picker("bookmark", "ctrl-t ctrl-b"),
        picker("branches", "ctrl-g b"),
        picker("quick", "alt-q"),
    ]);

    let bindings: Vec<&str> = script
        .lines()
        .filter(|line| line.starts_with("rex.bind("))
        .collect();
    assert_eq!(
        bindings,
        [
            r#"rex.bind("ctrl+t", key("ctrl-t"))"#,
            r#"rex.bind("ctrl+g", key("ctrl-g"))"#
        ]
    );
}

#[test]
fn an_alt_base_is_bound_by_rexs_name_for_it() {
    assert!(lua(&[picker("files", "alt-f x")]).contains(r#"rex.bind("alt+f", key("alt-f"))"#));
}

#[test]
fn the_script_runs_this_pwe_by_its_full_path_with_quotes_escaped() {
    assert!(lua(&[]).starts_with("local pwe = \"/opt/pwe/bin/pwe\"\n"));
    assert!(
        rex_lua("/odd \"dir\"\\pwe", &[]).starts_with("local pwe = \"/odd \\\"dir\\\"\\\\pwe\"\n")
    );
}

#[test]
fn the_script_reads_the_pane_through_the_terminal_block_and_opens_the_layer_pwe_answers() {
    let script = lua(&[]);

    assert!(script.contains(r#"local terminal = "com.superlogical.terminal""#));
    assert!(script.contains("rex.session.view{}"));
    assert!(script.contains(r#"rex.block.call(terminal, "process", { block_id = ctx.block_id })"#));
    assert!(script.contains(r#"rex.block.call(terminal, "size", { block_id = ctx.block_id })"#));
    assert!(script.contains(r#"rex.exec({ pwe, "rex", "key", base }"#));
    assert!(script.contains(r#"rex.call("session.new_layer", request)"#));
}
