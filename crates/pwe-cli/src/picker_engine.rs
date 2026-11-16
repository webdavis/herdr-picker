use std::env;
use std::path::Path;

use pwe_adapters::finder::{TerminalFinder, TtyTerminal};
use pwe_adapters::{FzfStepFinder, Placement, PluginConfig};
use pwe_application::{StepFinder, StepOutcome, StepSetup};
use pwe_domain::Engine;

use crate::picker_chrome::QuotedHelper;

pub(crate) struct EngineFinder<F, P> {
    pub(crate) engine: Engine,
    pub(crate) fzf: F,
    pub(crate) pwe: P,
}

impl<F: StepFinder, P: StepFinder<Error = F::Error>> StepFinder for EngineFinder<F, P> {
    type Error = F::Error;

    fn find(&self, setup: StepSetup) -> Result<Option<StepOutcome>, F::Error> {
        match (self.engine, setup.live) {
            (Engine::Pwe, false) => self.pwe.find(setup),
            (Engine::Pwe, true) | (Engine::Fzf, _) => self.fzf.find(setup),
        }
    }
}

pub(crate) fn engine_of(name: &str, config: &PluginConfig) -> Engine {
    config
        .pickers
        .iter()
        .find(|picker| picker.name == name)
        .map_or(Engine::Fzf, |picker| picker.engine)
}

pub(crate) fn step_finder<'a>(
    name: &str,
    config: &PluginConfig,
    cache: &'a Path,
    quoted: &'a QuotedHelper,
    quickfix: &'a Path,
    placement: Placement,
) -> EngineFinder<FzfStepFinder<'a>, TerminalFinder<'a, TtyTerminal>> {
    EngineFinder {
        engine: engine_of(name, config),
        fzf: FzfStepFinder {
            helper: quoted.helper(),
            cache,
            theme: config.theme,
            placement,
        },
        pwe: TerminalFinder {
            cache,
            theme: config.theme,
            quickfix,
            path_variable: env::var_os("PATH"),
            open_terminal: TtyTerminal::open,
        },
    }
}

#[cfg(test)]
mod tests;
