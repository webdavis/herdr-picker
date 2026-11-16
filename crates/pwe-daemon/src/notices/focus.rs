use pwe_application::PlacedPane;
use pwe_domain::{OwnerPlace, RexFocus, SessionSeen};

use super::NoticeDesk;

impl NoticeDesk {
    pub(super) fn owner_focus(&mut self) -> RexFocus {
        let rex_session = match self.parts.whereabouts.owner_place() {
            Ok(OwnerPlace::Front(SessionSeen::OnThisMac { rex_session, .. })) => rex_session,
            Ok(OwnerPlace::Front(_) | OwnerPlace::Background(_)) => return RexFocus::Background,
            Ok(OwnerPlace::Closed) => return RexFocus::Closed,
            Err(failure) => {
                self.report(failure);
                return RexFocus::Background;
            }
        };
        match self.parts.places.focused_pane(&rex_session) {
            Ok(pane) => RexFocus::Front(pane),
            Err(failure) => {
                self.report(format!("could not read the focused pane: {failure}"));
                RexFocus::Background
            }
        }
    }

    pub(super) fn placed(&mut self, block: &str) -> Option<PlacedPane> {
        match self.parts.places.placed_pane(block) {
            Ok(placed) => placed,
            Err(failure) => {
                self.report(format!("could not find the pane {block}: {failure}"));
                None
            }
        }
    }
}
