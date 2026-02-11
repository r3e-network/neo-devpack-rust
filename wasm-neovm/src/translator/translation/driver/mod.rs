use super::*;
use crate::config::validate_config;

mod exports;
mod finalize;
mod import_stub;
mod overlay;
mod parser;
mod reachability;
mod start;
mod state;

use state::DriverState;

pub fn translate_module(bytes: &[u8], contract_name: &str) -> Result<Translation> {
    let contract_name = crate::types::ContractName::try_new(contract_name)
        .ok_or_else(|| anyhow::anyhow!("contract name cannot be empty"))?;
    translate_with_config(bytes, TranslationConfig::new(contract_name))
}

pub fn translate_with_config(bytes: &[u8], config: TranslationConfig) -> Result<Translation> {
    validate_config(&config).context("invalid translation configuration")?;
    DriverState::new(config).translate(bytes)
}
