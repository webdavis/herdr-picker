use pwe_domain::Key;
use pwe_domain::finder::Ending;

pub(super) const BACK: &str = "esc";

pub const SWITCH_SOURCE: &str = "source";

pub const SAVE_QUICKFIX: &str = "quickfix";

pub const SEND_ROWS: &str = "send";

pub(crate) fn fzf_ending(word: &str) -> Option<Ending> {
    match word {
        BACK => Some(Ending::Back),
        SWITCH_SOURCE => Some(Ending::SwitchSource),
        SAVE_QUICKFIX => Some(Ending::SaveQuickfix),
        SEND_ROWS => Some(Ending::SendRows),
        key => Key::parse(key).map(Ending::Key),
    }
}
