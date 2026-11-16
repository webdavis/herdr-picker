use pwe_domain::Theme;
use serde::Deserialize;

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ThemeName {
    #[default]
    CatppuccinMocha,
}

impl From<ThemeName> for Theme {
    fn from(name: ThemeName) -> Self {
        match name {
            ThemeName::CatppuccinMocha => Self::CatppuccinMocha,
        }
    }
}
