use super::*;

mod guard;

const EXISTING: [&str; 3] = ["src/main.rs", "my notes.md", "src"];

fn underneath(kind: PaneKind) -> Underneath<'static> {
    Underneath {
        kind,
        cwd: "/work/repo",
        editor: "hx",
        is_path: &|word| EXISTING.contains(&word),
        is_program: &|word| ["docker", "just", "git"].contains(&word),
    }
}

fn over(kind: PaneKind, line: &str) -> Result<Handover, Refusal> {
    let instruction = parse_instruction(line).expect("an instruction");
    handover(&instruction, &underneath(kind))
}

fn typed(text: &str) -> Result<Handover, Refusal> {
    Ok(Handover::Type {
        text: text.to_string(),
        submit: false,
    })
}

fn sent(text: &str) -> Result<Handover, Refusal> {
    Ok(Handover::Type {
        text: text.to_string(),
        submit: true,
    })
}

fn opened(pane: NewPane, folder: Option<&str>, run: Option<&str>) -> Result<Handover, Refusal> {
    Ok(Handover::Open {
        pane,
        folder: folder.map(str::to_string),
        run: run.map(str::to_string),
    })
}

#[test]
fn a_shell_gets_inserts_typed_and_folders_and_commands_run() {
    assert_eq!(
        over(PaneKind::Shell, "insert\tsrc/main.rs"),
        typed("src/main.rs")
    );
    assert_eq!(
        over(PaneKind::Shell, "insert\tjust test"),
        typed("just test")
    );
    assert_eq!(over(PaneKind::Shell, "cd\tmy dir"), sent("cd 'my dir'"));
    assert_eq!(over(PaneKind::Shell, "run\tgit status"), sent("git status"));
    assert_eq!(over(PaneKind::Shell, "run\thx +3 a.rs"), sent("hx +3 a.rs"));
}

#[test]
fn claude_gets_paths_as_references_and_commands_behind_a_bang() {
    assert_eq!(
        over(PaneKind::Claude, "insert\tsrc/main.rs 'my notes.md'"),
        typed("@src/main.rs @my notes.md")
    );
    assert_eq!(over(PaneKind::Claude, "insert\tabc1234"), typed("abc1234"));
    assert_eq!(
        over(PaneKind::Claude, "insert\tdocker logs --tail 200 web"),
        typed("! docker logs --tail 200 web")
    );
    assert_eq!(
        over(PaneKind::Claude, "cd\tsrc"),
        sent("! cd /work/repo/src")
    );
    assert_eq!(over(PaneKind::Claude, "cd\t/srv"), sent("! cd /srv"));
    assert_eq!(
        over(PaneKind::Claude, "run\tgit status"),
        sent("! git status")
    );
}

#[test]
fn codex_gets_paths_as_typed_and_commands_behind_a_bang_like_claude() {
    assert_eq!(
        over(PaneKind::Codex, "insert\tsrc/main.rs 'my notes.md'"),
        typed("src/main.rs 'my notes.md'")
    );
    assert_eq!(
        over(PaneKind::Codex, "insert\tjust test"),
        typed("! just test")
    );
    assert_eq!(over(PaneKind::Codex, "run\tjust test"), sent("! just test"));
}

#[test]
fn a_command_that_needs_a_real_terminal_opens_a_new_pane_over_an_agent_only() {
    for command in [
        "hx +3 a.rs",
        "nvim a.rs",
        "ssh pi",
        "sudo vi /etc/hosts",
        "docker exec -it web sh",
        "TERM=xterm-256color vim a.rs",
    ] {
        let line = format!("run\t{command}");
        assert_eq!(
            over(PaneKind::Claude, &line),
            opened(NewPane::Right, None, Some(command)),
            "{command}"
        );
    }
    assert_eq!(over(PaneKind::Codex, "run\tdocker ps"), sent("! docker ps"));
}

#[test]
fn a_new_pane_instruction_opens_there_whatever_runs_underneath() {
    for kind in [PaneKind::Shell, PaneKind::Claude, PaneKind::Codex] {
        assert_eq!(
            over(kind.clone(), "left\trun\thx a.rs"),
            opened(NewPane::Left, None, Some("hx a.rs"))
        );
        assert_eq!(
            over(kind, "tab\tcd\tsrc"),
            opened(NewPane::Tab, Some("/work/repo/src"), None)
        );
    }
}

#[test]
fn an_agent_herdr_picker_does_not_know_and_an_insert_for_a_new_pane_are_refused() {
    assert_eq!(
        over(PaneKind::Agent("opencode".to_string()), "insert\tx"),
        Err(Refusal::UnknownAgent("opencode".to_string()))
    );
    assert_eq!(
        over(PaneKind::Shell, "below\tinsert\tx"),
        Err(Refusal::NothingToOpen)
    );
}

#[test]
fn an_instruction_parses_its_verb_and_any_new_pane_first() {
    assert_eq!(
        parse_instruction("above\trun\tls -la"),
        Some(Instruction {
            pane: Some(NewPane::Above),
            verb: Verb::Run,
            text: "ls -la".to_string(),
        })
    );
    assert_eq!(
        parse_instruction("insert\ta\tb"),
        Some(Instruction {
            pane: None,
            verb: Verb::Insert,
            text: "a\tb".to_string(),
        })
    );
    assert_eq!(parse_instruction("copy\tx"), None);
    assert_eq!(parse_instruction("left\tleft\trun\tx"), None);
}

#[test]
fn the_agent_a_pane_runs_names_its_kind() {
    let kinds: Vec<PaneKind> = [None, Some("claude"), Some("codex"), Some("pi")]
        .into_iter()
        .map(PaneKind::of)
        .collect();

    assert_eq!(
        kinds,
        vec![
            PaneKind::Shell,
            PaneKind::Claude,
            PaneKind::Codex,
            PaneKind::Agent("pi".to_string())
        ]
    );
}
