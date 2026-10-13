#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowLook {
    pub leaders: bool,
    pub stripes: bool,
}

impl Default for RowLook {
    fn default() -> Self {
        Self {
            leaders: true,
            stripes: false,
        }
    }
}
