use std::sync::atomic::{AtomicUsize, Ordering};

use pwe_domain::Engine;

use super::*;

fn engine_of(body: &str, name: &str) -> Result<Engine, String> {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    let call = CALLS.fetch_add(1, Ordering::SeqCst);
    let sandbox = TempDirectory::new(&format!("engine-{call}"));
    let path = sandbox.write(body);
    let config = read_config(&path).map_err(|refusal| refusal.to_string())?;
    Ok(config
        .pickers
        .iter()
        .find(|picker| picker.name == name)
        .expect("the picker")
        .engine)
}

#[test]
fn a_picker_table_chooses_the_pwe_engine() {
    assert_eq!(
        engine_of("[pickers.files]\nengine = \"pwe\"\n", "files"),
        Ok(Engine::Pwe)
    );
}

#[test]
fn a_picker_without_an_engine_runs_in_fzf() {
    assert_eq!(engine_of("", "files"), Ok(Engine::Fzf));
    assert_eq!(
        engine_of("[pickers.files]\nkey = \"ctrl-t z\"\n", "files"),
        Ok(Engine::Fzf)
    );
    assert_eq!(
        engine_of("[pickers.files]\nengine = \"fzf\"\n", "files"),
        Ok(Engine::Fzf)
    );
}

#[test]
fn a_second_table_runs_a_built_in_picker_in_the_pwe_engine_beside_the_first() {
    let body = "[pickers.files-pwe]\nbuiltin = \"files\"\nkey = \"ctrl-t p\"\nengine = \"pwe\"\n";

    assert_eq!(engine_of(body, "files-pwe"), Ok(Engine::Pwe));
    assert_eq!(engine_of(body, "files"), Ok(Engine::Fzf));
}

#[test]
fn an_unknown_engine_is_refused_naming_the_picker_the_field_and_the_accepted_words() {
    let refusal =
        engine_of("[pickers.files]\nengine = \"skim\"\n", "files").expect_err("a refused engine");

    assert!(
        refusal.contains(
            "the picker files sets engine to \"skim\", which is not one of \"fzf\" or \"pwe\""
        ),
        "{refusal}"
    );
}
