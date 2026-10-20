use std::time::{Duration, SystemTime};

use herdr_picker_adapters::{AgentSnapshot, SocketPanes, read_snapshot, recent_places};
use herdr_picker_application::JumpError;
use herdr_picker_domain::Place;

use crate::agent_hook::{now, refresh};
use crate::builtin::HerdrPlaces;

// The status hook keeps the saved list current; past this, a closed agent may linger in it.
const FRESH_FOR: Duration = Duration::from_secs(30);
const ASK_CAP: Duration = Duration::from_secs(2);

pub(super) struct HerdrView {
    pub(super) snapshot: AgentSnapshot,
    pub(super) recent: Vec<Place>,
    pub(super) tab: Option<String>,
    pub(super) herdr: String,
    pub(super) now: u64,
}

// What herdr runs: the hook's saved list while it is fresh, herdr's answer otherwise; none
// outside herdr.
pub(super) fn herdr_view(
    places: &HerdrPlaces,
    fresh: bool,
) -> Option<Result<HerdrView, JumpError>> {
    let socket = places.socket.as_ref()?;
    let saved = (!fresh)
        .then(|| read_snapshot(&places.snapshot, FRESH_FOR, SystemTime::now()))
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
