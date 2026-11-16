use super::*;
use pwe_domain::ProgramName;

fn agent_settings(name: &str, content: &str) -> Result<AgentSettings, ConfigError> {
    let sandbox = TempDirectory::new(name);
    read_config(&sandbox.write(content)).map(|config| config.agents)
}

#[test]
fn every_agent_program_is_read_every_fifteen_seconds_unless_told_otherwise() {
    let settings = agent_settings("agents-default", "").expect("a config");

    assert_eq!(
        settings.programs(),
        vec![
            ProgramName::ClaudeCode,
            ProgramName::Codex,
            ProgramName::Omp
        ]
    );
    assert_eq!(settings.refresh_seconds, 15);
}

#[test]
fn the_agents_table_chooses_the_programs_and_how_often_they_are_read() {
    let settings = agent_settings(
        "agents-set",
        "[agents]\nprograms = [\"codex\"]\nrefresh-seconds = 60\n",
    )
    .expect("a config");

    assert_eq!(settings.programs(), vec![ProgramName::Codex]);
    assert_eq!(settings.refresh_seconds, 60);
}

#[test]
fn an_unknown_agent_program_is_refused() {
    let refused = agent_settings("agents-unknown", "[agents]\nprograms = [\"cursor\"]\n");

    assert!(
        matches!(refused, Err(ConfigError::Unparseable { .. })),
        "{refused:?}"
    );
}
