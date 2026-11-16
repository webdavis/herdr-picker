use super::NoticeBoard;
use super::input::NoticeEffect;
use crate::notice::Pane;

impl NoticeBoard {
    pub(super) fn jump(&mut self, id: Option<&str>, now: u64) -> Vec<NoticeEffect> {
        let Some((id, pane)) = self.jump_target(id) else {
            return Vec::new();
        };
        let mut effects = vec![NoticeEffect::JumpTo(pane)];
        effects.extend(self.mark_read(&id, now));
        effects
    }

    fn jump_target(&self, id: Option<&str>) -> Option<(String, Pane)> {
        let target = match id {
            Some(id) => self.live.iter().find(|notice| notice.id == id).cloned(),
            None => self
                .waiting()
                .into_iter()
                .find(|notice| notice.pane.is_some()),
        }?;
        Some((target.id, target.pane?))
    }
}
