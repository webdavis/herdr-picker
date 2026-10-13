use herdr_workspace_jump_domain::Theme;

pub(super) struct Palette {
    pub(super) base: &'static str,
    pub(super) text: &'static str,
    pub(super) key: &'static str,
    pub(super) matched: &'static str,
    pub(super) open: &'static str,
    pub(super) faint: &'static str,
    pub(super) leader: &'static str,
    pub(super) stripe: &'static str,
    pub(super) bar: &'static str,
    pub(super) refusal: &'static str,
}

const MOCHA_BOLD_BLUE: &str = "\x1b[1;38;2;137;180;250m";

const CATPPUCCIN_MOCHA: Palette = Palette {
    base: "\x1b[48;2;30;30;46m",
    text: "\x1b[22;38;2;205;214;244m",
    key: MOCHA_BOLD_BLUE,
    matched: MOCHA_BOLD_BLUE,
    open: "\x1b[38;2;166;227;161m",
    faint: "\x1b[38;2;127;132;156m",
    leader: "\x1b[38;2;69;71;90m",
    stripe: "\x1b[48;2;33;33;52m",
    bar: "\x1b[48;2;54;54;70m",
    refusal: "\x1b[38;2;243;139;168m",
};

pub(super) fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::CatppuccinMocha => &CATPPUCCIN_MOCHA,
    }
}
