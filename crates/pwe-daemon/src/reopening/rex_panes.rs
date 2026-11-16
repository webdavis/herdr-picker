use pwe_adapters::{PaneHost, RexPanes};
use pwe_protocol::PaneFront;

use super::pass::{PaneNow, RestoredPanes};

impl RestoredPanes for RexPanes {
    fn pane_now(&mut self, block: &str) -> PaneNow {
        match self.front(block) {
            Ok(PaneFront::TheShell { folder }) => PaneNow::ShellWaits { folder },
            Ok(PaneFront::AnotherProgram) => PaneNow::RunsAProgram,
            Ok(PaneFront::NotKnown) | Err(_) => PaneNow::NotReady,
        }
    }

    fn type_line(&mut self, block: &str, line: &str) {
        let typed = self
            .type_text(block, line)
            .and_then(|()| self.press_enter(block));
        if let Err(failure) = typed {
            eprintln!("pwe: could not reopen the agent in {block}: {failure}");
        }
    }
}
