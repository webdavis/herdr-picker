use herdr_picker_domain::{Action, PickerRow, Preview};

use super::*;
use crate::builtin::rows::key;

const PS: &str = "    1     0 root             /sbin/launchd
  722     1 stephen          /usr/libexec/rapportd
  900   722 stephen          /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
garbage line
";

const LSOF: &str = "p722
crapportd
f11
n*:63630
TST=LISTEN
TQR=0
f12
n192.168.1.20:52000->17.0.0.1:443
TST=ESTABLISHED
p900
cGoogle
f30
nlocalhost:9222
";

#[test]
fn each_process_is_a_row_with_its_pid_program_and_the_sockets_it_holds() {
    let rows = process_rows(PS, LSOF);

    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[1],
        PickerRow {
            label: "\u{1b}[38;2;249;226;175m    722\u{1b}[0m  rapportd  \u{1b}[2m/usr/libexec\u{1b}[0m  \u{1b}[38;2;148;226;213m*:63630 LISTEN; 192.168.1.20:52000->17.0.0.1:443 ESTABLISHED\u{1b}[0m".to_string(),
            preview: Some(Preview::Text(
                "rapportd\n\nPID         722\nParent PID  1\nUser        stephen\nProgram     /usr/libexec/rapportd\n\nOpen connections and ports\n*:63630 LISTEN\n192.168.1.20:52000->17.0.0.1:443 ESTABLISHED".to_string()
            )),
            enter: Action::Insert("722".to_string()),
            keys: vec![(key("alt-d"), Action::Insert("lsof -nP -a -p 722 -i".to_string()))],
            group: None,
        }
    );
}

#[test]
fn a_process_without_sockets_shows_no_connections_and_keeps_spaces_in_its_path() {
    let rows = process_rows(PS, "");

    assert_eq!(
        rows[2].label,
        "\u{1b}[38;2;249;226;175m    900\u{1b}[0m  Google Chrome  \u{1b}[2m/Applications/Google Chrome.app/Contents/MacOS\u{1b}[0m"
    );
    assert_eq!(
        rows[0].preview,
        Some(Preview::Text(
            "launchd\n\nPID         1\nParent PID  0\nUser        root\nProgram     /sbin/launchd"
                .to_string()
        ))
    );
}
