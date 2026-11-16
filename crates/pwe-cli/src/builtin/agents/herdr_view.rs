use std::time::{Duration, SystemTime};

use pwe_adapters::{AgentSnapshot, SocketPanes, read_snapshot, recent_places};
use pwe_application::JumpError;
use pwe_domain::Place;

use crate::agent_hook::{now, refresh};
use crate::builtin::HerdrPlaces;

const SAVED_LIST_TRUSTED_FOR: Duration = Duration::from_secs(30);
const ASK_CAP: Duration = Duration::from_secs(2);

pub(super) struct HerdrView {
    pub(super) snapshot: AgentSnapshot,
    pub(super) recent: Vec<Place>,
    pub(super) tab: Option<String>,
    pub(super) herdr: String,
    pub(super) now: u64,
}

pub(super) fn herdr_view(
    places: &HerdrPlaces,
    fresh: bool,
) -> Option<Result<HerdrView, JumpError>> {
    let socket = places.socket.as_ref()?;
    let saved = (!fresh)
        .then(|| read_snapshot(&places.snapshot, SAVED_LIST_TRUSTED_FOR, SystemTime::now()))
        .flatten();
    let snapshot = match saved {
        Some(snapshot) => Ok(snapshot),
        None => refresh(
            &mut SocketPanes::at(socket, ASK_CAP),
            &places.snapshot,
            "",
            now(),
        ),
    };
    Some(snapshot.map(|snapshot| HerdrView {
        snapshot,
        recent: recent_places(&places.places),
        tab: places.tab.clone(),
        herdr: places.bin.clone(),
        now: now(),
    }))
}
