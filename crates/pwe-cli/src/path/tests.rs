use super::*;

#[test]
fn expand_tilde_joins_a_leading_tilde_onto_home() {
    assert_eq!(
        expand_tilde("~/workspaces/Ivy", Some("/Users/me")),
        Ok(PathBuf::from("/Users/me/workspaces/Ivy"))
    );
    assert_eq!(
        expand_tilde("~", Some("/Users/me")),
        Ok(PathBuf::from("/Users/me"))
    );
}
#[test]
fn expand_tilde_leaves_an_absolute_or_relative_path_alone() {
    assert_eq!(
        expand_tilde("/opt/project", Some("/Users/me")),
        Ok(PathBuf::from("/opt/project"))
    );
    assert_eq!(
        expand_tilde("relative/path", Some("/Users/me")),
        Ok(PathBuf::from("relative/path"))
    );
}
#[test]
fn a_tilde_anywhere_but_the_front_is_an_ordinary_character() {
    assert_eq!(
        expand_tilde("/opt/~backup", Some("/Users/me")),
        Ok(PathBuf::from("/opt/~backup"))
    );
}
#[test]
fn expand_tilde_refuses_another_users_home_and_a_missing_home() {
    assert!(expand_tilde("~other/project", Some("/Users/me")).is_err());
    assert!(expand_tilde("~/project", None).is_err());
    assert!(expand_tilde("~/project", Some("")).is_err());
}
