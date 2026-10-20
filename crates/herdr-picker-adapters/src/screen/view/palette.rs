use herdr_picker_domain::Theme;

pub(crate) struct Palette {
    pub(crate) base: &'static str,
    pub(crate) text: &'static str,
    pub(crate) key: &'static str,
    pub(crate) matched: &'static str,
    pub(crate) open: &'static str,
    pub(crate) faint: &'static str,
    pub(crate) leader: &'static str,
    pub(crate) stripe: &'static str,
    pub(crate) bar: &'static str,
    pub(crate) refusal: &'static str,
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

pub(crate) fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::CatppuccinMocha => &CATPPUCCIN_MOCHA,
    }
}
