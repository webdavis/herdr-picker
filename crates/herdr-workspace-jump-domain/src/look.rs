#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RowLook {
    pub leaders: bool,
    pub stripes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    CatppuccinMocha,
}
