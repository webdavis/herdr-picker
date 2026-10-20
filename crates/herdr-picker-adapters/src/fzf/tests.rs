use herdr_picker_domain::bindable_keys;

use super::*;

const CHROME: Chrome = Chrome {
    helper: Helper {
        program: "'/bin/herdr-picker'",
        cache: "'/state/rows'",
    },
    header: "Git Branches",
    description: "Branches of this repository",
    query: "ma",
    live: false,
};

fn row_key_binds() -> impl Iterator<Item = String> {
    bindable_keys().map(|key| {
        format!(
            "--bind={0}:transform('/bin/herdr-picker' view '/state/rows' key {1} {{+1}})",
            key.name(),
            herdr_picker_protocol::shell_word(key.name())
        )
    })
}

#[test]
fn a_key_the_shell_would_split_reaches_the_helper_quoted() {
    let arguments = fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Inline);

    assert!(
        arguments.contains(
            &"--bind=alt-;:transform('/bin/herdr-picker' view '/state/rows' key 'alt-;' {+1})"
                .to_string()
        )
    );
}

#[test]
fn fzf_draws_the_picker_like_the_pick_popup_in_the_theme_colors() {
    let arguments = fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Inline);
    let bound_keys = bindable_keys().count();
    let (before, rest) = arguments.split_at(arguments.len() - bound_keys - 1);
    assert_eq!(
        rest[..bound_keys].to_vec(),
        row_key_binds().collect::<Vec<_>>()
    );
    assert_eq!(
        rest.last().map(String::as_str),
        Some(
            "--color=bg:#1e1e2e,bg+:#363646,fg:#cdd6f4,fg+:#cdd6f4,hl:#89b4fa:bold,hl+:#89b4fa:bold,info:#7f849c,prompt:#cdd6f4,query:#cdd6f4,pointer:#89b4fa,gutter:#1e1e2e,border:#45475a,preview-bg:#1e1e2e,header:#7f849c,footer:#7f849c"
        )
    );
    assert_eq!(
        before,
        vec![
            "--ansi",
            "--delimiter=\t",
            "--with-nth=2..",
            "--layout=reverse",
            "--info=inline-right",
            "--info-command=echo $FZF_MATCH_COUNT",
            "--prompt= /",
            "--pointer= ",
            "--highlight-line",
            "--no-separator",
            "--no-scrollbar",
            "--border=none",
            "--list-border=none",
            "--input-border=none",
            "--multi",
            "--print-query",
            "--preview='/bin/herdr-picker' preview '/state/rows' {1}",
            "--preview-window=right,50%,border-left",
            "--header=Git Branches",
            "--query=ma",
            "--footer=Branches of this repository",
            "--bind=load:transform-header('/bin/herdr-picker' header '/state/rows')+transform('/bin/herdr-picker' view '/state/rows' load)",
            "--bind=focus:transform-footer('/bin/herdr-picker' footer '/state/rows' {1})",
            "--bind=f1:preview('/bin/herdr-picker' help '/state/rows' {1})",
            "--bind=ctrl-space:transform('/bin/herdr-picker' view '/state/rows' choose)",
            "--bind=esc:transform('/bin/herdr-picker' view '/state/rows' back)",
            "--bind=ctrl-q:transform('/bin/herdr-picker' view '/state/rows' quickfix {+1})",
            "--bind=alt-o:transform('/bin/herdr-picker' view '/state/rows' send)",
        ]
    );
}

#[test]
fn every_key_a_row_may_claim_asks_herdr_picker_what_it_does_to_the_marked_rows() {
    let arguments = fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Inline);

    assert!(
        arguments.contains(
            &"--bind=enter:transform('/bin/herdr-picker' view '/state/rows' key enter {+1})"
                .to_string()
        )
    );
    assert!(arguments.contains(
        &"--bind=alt-enter:transform('/bin/herdr-picker' view '/state/rows' key alt-enter {+1})"
            .to_string()
    ));
}

#[test]
fn a_picker_without_a_description_has_no_footer() {
    let chrome = Chrome {
        description: "",
        ..CHROME
    };

    let arguments = fzf_arguments(Theme::CatppuccinMocha, &chrome, Placement::Inline);

    assert!(
        !arguments
            .iter()
            .any(|argument| argument.starts_with("--footer"))
    );
}

#[test]
fn in_a_popup_fzf_fills_the_whole_popup_whatever_height_the_defaults_ask_for() {
    let arguments = fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Popup);
    assert_eq!(arguments.last().map(String::as_str), Some("--height=100%"));
}

#[test]
fn inline_fzf_keeps_the_height_the_defaults_ask_for() {
    let arguments = fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Inline);
    assert!(
        !arguments
            .iter()
            .any(|argument| argument.starts_with("--height"))
    );
}

#[test]
fn a_live_step_leaves_matching_to_its_command_and_runs_it_again_on_every_change() {
    let live = Chrome {
        live: true,
        ..CHROME
    };

    let arguments = fzf_arguments(Theme::CatppuccinMocha, &live, Placement::Inline);
    let still = fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Inline);

    assert!(arguments.contains(&"--disabled".to_string()));
    assert!(
        arguments.contains(
            &"--bind=change:reload('/bin/herdr-picker' live '/state/rows' {q})".to_string()
        )
    );
    assert!(!still.contains(&"--disabled".to_string()));
    assert!(
        !still
            .iter()
            .any(|argument| argument.starts_with("--bind=change:"))
    );
}

#[test]
fn a_key_keeps_the_editing_or_moving_fzf_gives_it_unbound() {
    let kept: Vec<Option<&str>> = ["ctrl-a", "ctrl-e", "ctrl-k", "alt-d", "ctrl-o", "f2"]
        .into_iter()
        .map(unbound_action)
        .collect();

    assert_eq!(
        kept,
        vec![
            Some("beginning-of-line"),
            Some("end-of-line"),
            Some("up"),
            Some("kill-word"),
            None,
            None
        ]
    );
}
