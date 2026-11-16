use pwe_application::{Whereabouts, WhereaboutsError};
use pwe_domain::{OwnerPlace, SessionSeen};

use super::RexWhereabouts;
use crate::fake_program::{Answer, FakePrograms};

const APP_NAME: &str = "Example Terminal";

fn osascript_printing(stdout: &str) -> FakePrograms {
    FakePrograms::new().answering(
        "osascript",
        Answer {
            stdout,
            stderr: "",
            exit_code: 0,
        },
    )
}

fn place_reported_by(programs: &FakePrograms) -> Result<OwnerPlace, WhereaboutsError> {
    RexWhereabouts::new(APP_NAME, Some(programs.path_variable())).owner_place()
}

fn this_mac_session() -> SessionSeen {
    SessionSeen::OnThisMac {
        rex_session: "session:00000000-0000-7000-8000-00000000a001".to_string(),
        name: "example session".to_string(),
    }
}

const SESSION_ON_THIS_MAC: &str = r#""session":{"id":"host:00000000-0000-7000-8000-00000000e001/session:00000000-0000-7000-8000-00000000a001","name":"example session","remote":false}"#;

#[test]
fn a_closed_rex_reads_as_closed() {
    let programs = osascript_printing(r#"{"running":false}"#);

    assert_eq!(place_reported_by(&programs), Ok(OwnerPlace::Closed));
}

#[test]
fn the_app_name_reaches_osascript_as_an_argument() {
    let programs = osascript_printing(r#"{"running":false}"#);

    let _ = place_reported_by(&programs);

    assert_eq!(
        programs.arguments("osascript"),
        Some(vec![
            "-l".to_string(),
            "JavaScript".to_string(),
            "-".to_string(),
            APP_NAME.to_string()
        ])
    );
    let script = programs.input("osascript");
    assert!(
        script.contains("running()") && !script.contains(APP_NAME),
        "{script}"
    );
}

#[test]
fn a_frontmost_rex_reads_as_front_with_its_session_without_its_host() {
    let programs = osascript_printing(&format!(
        r#"{{"running":true,"frontmost":true,{SESSION_ON_THIS_MAC}}}"#
    ));

    assert_eq!(
        place_reported_by(&programs),
        Ok(OwnerPlace::Front(this_mac_session()))
    );
}

#[test]
fn a_rex_behind_another_app_reads_as_background_with_its_session() {
    let programs = osascript_printing(&format!(
        r#"{{"running":true,"frontmost":false,{SESSION_ON_THIS_MAC}}}"#
    ));

    assert_eq!(
        place_reported_by(&programs),
        Ok(OwnerPlace::Background(this_mac_session()))
    );
}

#[test]
fn a_session_on_another_host_reads_as_elsewhere() {
    let programs = osascript_printing(
        r#"{"running":true,"frontmost":true,"session":{"id":"host:remote/session:x","name":"remote example","remote":true}}"#,
    );

    assert_eq!(
        place_reported_by(&programs),
        Ok(OwnerPlace::Front(SessionSeen::Elsewhere {
            name: "remote example".to_string()
        }))
    );
}

#[test]
fn a_window_showing_no_session_reads_as_no_session() {
    let programs = osascript_printing(r#"{"running":true,"frontmost":true,"session":null}"#);

    assert_eq!(
        place_reported_by(&programs),
        Ok(OwnerPlace::Front(SessionSeen::NoSession))
    );
}

#[test]
fn a_refused_automation_permission_reads_as_refused() {
    let programs = FakePrograms::new().answering(
        "osascript",
        Answer {
            stdout: "",
            stderr: "execution error: Error: Not authorized to send Apple events to Example Terminal. (-1743)",
            exit_code: 1,
        },
    );

    assert_eq!(
        place_reported_by(&programs),
        Err(WhereaboutsError::PermissionRefused {
            app_name: APP_NAME.to_string()
        })
    );
}

#[test]
fn another_osascript_failure_keeps_its_reason() {
    let programs = FakePrograms::new().answering(
        "osascript",
        Answer {
            stdout: "",
            stderr: "execution error: Error: Application can't be found. (-2700)",
            exit_code: 1,
        },
    );

    let Err(WhereaboutsError::Unreadable(reason)) = place_reported_by(&programs) else {
        panic!("a failed osascript is unreadable");
    };
    assert!(reason.contains("Application can't be found"), "{reason}");
}

#[test]
fn a_missing_osascript_is_named() {
    let programs = FakePrograms::new();

    assert_eq!(
        place_reported_by(&programs),
        Err(WhereaboutsError::ProgramMissing("osascript".to_string()))
    );
}
