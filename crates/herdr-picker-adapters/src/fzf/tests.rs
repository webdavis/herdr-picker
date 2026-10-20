use super::*;

const CHROME: Chrome = Chrome {
    helper: Helper {
        program: "'/bin/herdr-picker'",
        cache: "'/state/rows'",
    },
    name: "git-branches",
    description: "Branches of this repository",
};

#[test]
fn fzf_draws_the_picker_like_the_pick_popup_in_the_theme_colors() {
    assert_eq!(
        fzf_arguments(Theme::CatppuccinMocha, &CHROME, Placement::Inline),
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
            "--preview='/bin/herdr-picker' preview '/state/rows' {1}",
            "--preview-window=right,50%,border-left",
            "--header=Git Branches",
            "--footer=Branches of this repository",
            "--bind=load:transform-header('/bin/herdr-picker' header '/state/rows' git-branches)+transform('/bin/herdr-picker' view '/state/rows' load)",
            "--bind=ctrl-space:transform('/bin/herdr-picker' view '/state/rows' choose)",
            "--bind=enter:transform('/bin/herdr-picker' view '/state/rows' enter {1})",
            "--bind=esc:transform('/bin/herdr-picker' view '/state/rows' back)",
            "--color=bg:#1e1e2e,bg+:#363646,fg:#cdd6f4,fg+:#cdd6f4,hl:#89b4fa:bold,hl+:#89b4fa:bold,info:#7f849c,prompt:#cdd6f4,query:#cdd6f4,pointer:#89b4fa,gutter:#1e1e2e,border:#45475a,preview-bg:#1e1e2e,header:#7f849c,footer:#7f849c",
        ]
    );
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
