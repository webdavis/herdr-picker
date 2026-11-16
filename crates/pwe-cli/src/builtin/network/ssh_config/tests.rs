use std::fs;

use super::*;
use crate::builtin::tests::Tree;

#[test]
fn static_aliases_carry_their_settings_and_includes_are_followed_once() {
    let tree = Tree::new("ssh-config", &[]);
    fs::create_dir_all(tree.0.join("conf.d")).expect("a folder");
    fs::write(
        tree.0.join("config"),
        "# mine\nInclude conf.d/*.conf\nHost nas backup # two names\n  HostName 192.168.1.5\n  User=stephen\n  Port 2222\nHost *.example.com bad;name !skip\n  User nobody\nMatch host x\n  User lost\nInclude config\nInclude ~/more\n",
    )
    .expect("a config");
    fs::write(tree.0.join("conf.d/pi.conf"), "Host pi\n  ProxyJump nas\n").expect("an include");
    fs::write(tree.0.join("conf.d/skip.txt"), "Host never\n").expect("a file");
    fs::create_dir_all(tree.0.join("home")).expect("a home");
    fs::write(tree.0.join("home/more"), "Host home-box\n").expect("a home include");

    let hosts = ssh_hosts(&tree.0.join("config"), Some(&tree.0.join("home")));

    let aliases: Vec<&str> = hosts.iter().map(|host| host.alias.as_str()).collect();
    assert_eq!(aliases, ["pi", "nas", "backup", "home-box"]);
    assert_eq!(hosts[0].settings, ["Proxyjump: nas"]);
    assert_eq!(
        hosts[1].settings,
        ["Hostname: 192.168.1.5", "User: stephen", "Port: 2222"]
    );
    assert_eq!(hosts[2].settings, hosts[1].settings);
    assert_eq!(hosts[1].file, tree.0.join("config"));
}

#[test]
fn a_missing_config_has_no_hosts() {
    assert!(ssh_hosts(Path::new("/nonexistent/ssh/config"), None).is_empty());
}

#[test]
fn wildcards_match_whole_names() {
    assert!(matches("*.conf", "pi.conf"));
    assert!(matches("p?.conf", "pi.conf"));
    assert!(!matches("*.conf", "pi.conf.bak"));
    assert!(matches("*", "anything"));
}
