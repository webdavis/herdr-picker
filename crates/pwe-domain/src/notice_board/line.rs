use crate::notice::Notice;

pub(super) fn notice_line(notice: &Notice) -> String {
    notice.text.clone()
}
