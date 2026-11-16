use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pwe_application::{
    DesktopNotifier, DesktopNotifierError, JumpError, NoticeFloat, NoticeScreen, PanePlaces,
    PlacedPane, Whereabouts, WhereaboutsError,
};
use pwe_domain::{NoticeSettings, OwnerPlace, Pane, SessionSeen};

use crate::NoticeParts;

#[derive(Clone, Default)]
pub(crate) struct Calls(Arc<Mutex<Vec<String>>>);

impl Calls {
    pub(crate) fn push(&self, call: String) {
        if let Ok(mut calls) = self.0.lock() {
            calls.push(call);
        }
    }

    pub(crate) fn taken(&self) -> Vec<String> {
        self.0
            .lock()
            .map(|mut calls| std::mem::take(&mut *calls))
            .unwrap_or_default()
    }
}

impl Write for Calls {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.push(String::from_utf8_lossy(bytes).trim_end().to_string());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) const SESSION: &str = "session:00000000-0000-4000-8000-0000000000a1";
pub(crate) const GONE_PANE: &str = "block:00000000-0000-4000-8000-0000000000b0";

pub(crate) fn pane(block: &str) -> Pane {
    Pane {
        rex_session: SESSION.to_string(),
        tab: "window:00000000-0000-4000-8000-0000000000d1".to_string(),
        block: block.to_string(),
    }
}

pub(crate) struct FakeScreen {
    pub(crate) calls: Calls,
    pub(crate) fails: bool,
}

impl FakeScreen {
    fn answer(&self, call: String) -> Result<(), JumpError> {
        self.calls.push(call);
        match self.fails {
            true => Err(JumpError::Unreachable("rex is gone".to_string())),
            false => Ok(()),
        }
    }
}

impl NoticeScreen for FakeScreen {
    fn show(&mut self, notice: &NoticeFloat) -> Result<(), JumpError> {
        self.answer(format!(
            "show {} in {} slot {} until {}: {}",
            notice.float, notice.rex_session, notice.slot, notice.until, notice.line
        ))
    }

    fn relabel(&mut self, float: &str, line: &str) -> Result<(), JumpError> {
        self.answer(format!("relabel {float}: {line}"))
    }

    fn close(&mut self, float: &str) -> Result<(), JumpError> {
        self.answer(format!("close {float}"))
    }
}

pub(crate) struct FakeNotifier(pub(crate) Calls);

impl DesktopNotifier for FakeNotifier {
    fn notify(&self, title: &str, text: &str) -> Result<(), DesktopNotifierError> {
        self.0.push(format!("macos {title}: {text}"));
        Ok(())
    }
}

pub(crate) struct FakeWhereabouts {
    pub(crate) calls: Calls,
    pub(crate) place: Arc<Mutex<OwnerPlace>>,
}

pub(crate) fn in_front_of(session: &str) -> OwnerPlace {
    OwnerPlace::Front(SessionSeen::OnThisMac {
        rex_session: session.to_string(),
        name: "invented".to_string(),
    })
}

impl Whereabouts for FakeWhereabouts {
    fn owner_place(&self) -> Result<OwnerPlace, WhereaboutsError> {
        self.calls.push("where".to_string());
        self.place
            .lock()
            .map(|place| place.clone())
            .map_err(|_| WhereaboutsError::Unreadable("poisoned".to_string()))
    }
}

pub(crate) struct FakePlaces {
    pub(crate) focused: Arc<Mutex<Pane>>,
}

impl PanePlaces for FakePlaces {
    fn focused_pane(&mut self, rex_session: &str) -> Result<Pane, JumpError> {
        let focused = self
            .focused
            .lock()
            .map(|pane| pane.clone())
            .map_err(|_| JumpError::Malformed("poisoned".to_string()))?;
        Ok(Pane {
            rex_session: rex_session.to_string(),
            ..focused
        })
    }

    fn placed_pane(&mut self, block: &str) -> Result<Option<PlacedPane>, JumpError> {
        Ok((block != GONE_PANE).then(|| PlacedPane {
            pane: pane(block),
            tab_label: "api".to_string(),
        }))
    }
}

pub(crate) const OWNER_PANE: &str = "block:00000000-0000-4000-8000-0000000000b1";

pub(crate) struct Recorders {
    pub(crate) screen: Calls,
    pub(crate) macos: Calls,
    pub(crate) asked: Calls,
    pub(crate) log: Calls,
    pub(crate) place: Arc<Mutex<OwnerPlace>>,
}

pub(crate) fn fake_parts(inbox: PathBuf, screen_fails: bool) -> (NoticeParts, Recorders) {
    let recorders = Recorders {
        screen: Calls::default(),
        macos: Calls::default(),
        asked: Calls::default(),
        log: Calls::default(),
        place: Arc::new(Mutex::new(in_front_of(SESSION))),
    };
    let parts = NoticeParts {
        settings: NoticeSettings::default(),
        screen: Box::new(FakeScreen {
            calls: recorders.screen.clone(),
            fails: screen_fails,
        }),
        notifier: Box::new(FakeNotifier(recorders.macos.clone())),
        whereabouts: Box::new(FakeWhereabouts {
            calls: recorders.asked.clone(),
            place: Arc::clone(&recorders.place),
        }),
        places: Box::new(FakePlaces {
            focused: Arc::new(Mutex::new(pane(OWNER_PANE))),
        }),
        inbox,
        log: Box::new(recorders.log.clone()),
    };
    (parts, recorders)
}
