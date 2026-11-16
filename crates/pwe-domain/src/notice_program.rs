#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeEvent {
    TimeIs(u64),
    KeyOrClick,
    Resized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeStep {
    Wait { seconds: u64 },
    Paint,
    End,
}

pub fn notice_step(until: u64, event: NoticeEvent) -> NoticeStep {
    match event {
        NoticeEvent::KeyOrClick => NoticeStep::End,
        NoticeEvent::Resized => NoticeStep::Paint,
        NoticeEvent::TimeIs(now) if now >= until => NoticeStep::End,
        NoticeEvent::TimeIs(now) => NoticeStep::Wait {
            seconds: until - now,
        },
    }
}

#[cfg(test)]
mod tests;
