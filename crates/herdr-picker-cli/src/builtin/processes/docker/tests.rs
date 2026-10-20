use herdr_picker_domain::Action;

use super::*;
use crate::builtin::rows::key;

const DOCKER: &str = r#"{"ID":"abc123","Names":"web","Image":"nginx","State":"running","Status":"Up 2 hours","Ports":"0.0.0.0:80->80/tcp"}
{"ID":"def456","Names":"old","Image":"redis","State":"exited","Status":"Exited (0) 3 days ago"}
"#;

#[test]
fn a_container_prepares_inspect_and_offers_its_logs_and_a_shell() {
    let mut seen = Vec::new();
    let rows = docker_rows(DOCKER, &mut seen).expect("containers");

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].label,
        "web  \u{1b}[38;2;166;227;161mrunning\u{1b}[0m"
    );
    assert_eq!(
        rows[0].enter,
        Action::Insert(
            "docker inspect --format '{{.Name}}  {{.State.Status}}  {{.Config.Image}}' abc123"
                .to_string()
        )
    );
    assert_eq!(
        rows[0].keys[1..],
        [
            (
                key("alt-d"),
                Action::Insert("docker logs --tail 200 abc123".to_string())
            ),
            (
                key("alt-e"),
                Action::Insert("docker exec -it abc123 sh".to_string())
            ),
        ]
    );
    assert_eq!(
        rows[1].label,
        "old  \u{1b}[38;2;243;139;168mexited\u{1b}[0m"
    );
    assert_eq!(seen, ["abc123", "def456"]);
}

#[test]
fn compose_containers_already_listed_by_docker_ps_are_not_repeated() {
    let mut seen = vec!["abc123".to_string()];
    let compose = r#"[{"ID":"abc123","Name":"web"},{"ID":"fff999","Name":"db","State":"running"}]"#;

    let rows = docker_rows(compose, &mut seen).expect("containers");

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].label,
        "db  \u{1b}[38;2;166;227;161mrunning\u{1b}[0m"
    );
}
