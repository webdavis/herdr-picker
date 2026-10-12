use super::*;

mod refusals;

const FIVE_WORKSPACES_DECLARED_OUT_OF_ORDER: &str = r#"
[workspaces]
netpulse = "/opt/netpulse"
Ivy = "~/workspaces/Ivy"
"casually-concerned" = "~/repos/casually-concerned"
homelab = "/srv/homelab"
"justdavis-ansible" = "~/repos/justdavis-ansible"
"#;

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn write(&self, content: &str) -> PathBuf {
        let path = self.0.join("config.toml");
        fs::write(&path, content).expect("a sandbox file");
        path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn refusal_for(name: &str, content: &str) -> (ConfigError, String) {
    let sandbox = TempDirectory::new(name);
    let path = sandbox.write(content);
    let refusal = read_jump_targets(&path).expect_err("the config is refused");
    let rendered = refusal.to_string();
    assert!(
        rendered.contains(&path.display().to_string()),
        "the refusal names the file: {rendered}"
    );
    assert!(!rendered.contains('\n'), "one sentence: {rendered}");
    (refusal, rendered)
}

#[test]
fn config_file_prefers_the_directory_herdr_hands_the_plugin() {
    assert_eq!(
        config_file(
            Some("/run/herdr/jump"),
            Some("/home/me/xdg"),
            Some("/home/me")
        ),
        PathBuf::from("/run/herdr/jump/config.toml")
    );
}

#[test]
fn config_file_falls_back_to_the_documented_path_under_xdg_then_home() {
    assert_eq!(
        config_file(None, Some("/home/me/xdg"), Some("/home/me")),
        PathBuf::from("/home/me/xdg/herdr/plugins/config/herdr-workspace-jump/config.toml")
    );
    assert_eq!(
        config_file(Some(""), None, Some("/home/me")),
        PathBuf::from("/home/me/.config/herdr/plugins/config/herdr-workspace-jump/config.toml")
    );
    assert_eq!(
        config_file(None, Some(""), Some("/home/me")),
        PathBuf::from("/home/me/.config/herdr/plugins/config/herdr-workspace-jump/config.toml")
    );
}

#[test]
fn read_jump_targets_orders_five_declared_workspaces_by_label() {
    let sandbox = TempDirectory::new("ordered");
    let path = sandbox.write(FIVE_WORKSPACES_DECLARED_OUT_OF_ORDER);

    let targets = read_jump_targets(&path).expect("a readable config");

    let labels: Vec<&str> = targets.iter().map(|target| target.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "Ivy",
            "casually-concerned",
            "homelab",
            "justdavis-ansible",
            "netpulse"
        ]
    );
    let ids: Vec<&str> = targets
        .iter()
        .map(|target| target.action_id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            "jump_ivy",
            "jump_casually_concerned",
            "jump_homelab",
            "jump_justdavis_ansible",
            "jump_netpulse"
        ]
    );
    assert_eq!(
        targets[0].directory, "~/workspaces/Ivy",
        "the tilde stays for the jump verb to expand"
    );
}

#[test]
fn read_jump_targets_accepts_a_table_declaring_a_directory_and_a_pick_key() {
    let sandbox = TempDirectory::new("keyed");
    let path = sandbox.write(
        "[workspaces]\nhomelab = \"/srv/homelab\"\nIvy = { dir = \"~/workspaces/Ivy\", key = \"v\" }\n",
    );

    let targets = read_jump_targets(&path).expect("a readable config");

    assert_eq!(targets[0].label, "Ivy");
    assert_eq!(targets[0].directory, "~/workspaces/Ivy");
    assert_eq!(targets[0].pick_key, 'v');
    assert_eq!(
        targets[1].pick_key, 'h',
        "a bare string still defaults its key"
    );
}
