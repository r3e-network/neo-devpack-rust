// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Configuration validation
//!
//! This module provides validation logic for translation configurations,
//! ensuring that all settings are valid and consistent.

use thiserror::Error;

use super::options::{BehaviorConfig, LogLevel, TranslationConfig};

/// Errors that can occur during configuration validation
#[derive(Debug, Error)]
pub enum ConfigValidationError {
    /// Invalid contract name
    #[error("Invalid contract name: {0}")]
    InvalidContractName(String),

    /// Invalid memory page count
    #[error("Invalid memory page count: {0} (must be between 1 and {1})")]
    InvalidMemoryPages(u32, u32),

    /// Invalid table size
    #[error("Invalid table size: {0} (must be between 0 and {1})")]
    InvalidTableSize(u32, u32),

    /// Invalid stack size
    #[error("Invalid stack size: {0} (must be at least {1})")]
    InvalidStackSize(u32, u32),

    /// Inconsistent feature flags
    #[error("Inconsistent feature flags: {0}")]
    InconsistentFeatures(String),

    /// Invalid output configuration
    #[error("Invalid output configuration: {0}")]
    InvalidOutput(String),
}

/// Result type for configuration validation
pub type ValidationResult<T> = Result<T, ConfigValidationError>;

/// Validate a complete translation configuration
pub fn validate_config(config: &TranslationConfig) -> ValidationResult<()> {
    // Validate contract name
    if config.contract_name.as_str().is_empty() {
        return Err(ConfigValidationError::InvalidContractName(
            "Contract name cannot be empty".to_string(),
        ));
    }

    if config.contract_name.as_str().len() > 256 {
        return Err(ConfigValidationError::InvalidContractName(
            "Contract name too long (max 256 chars)".to_string(),
        ));
    }

    // Validate behavior config
    validate_behavior(&config.behavior)?;

    // Validate output config
    validate_output(&config.output)?;

    Ok(())
}

/// Validate behavior configuration
pub fn validate_behavior(config: &BehaviorConfig) -> ValidationResult<()> {
    const MAX_MEMORY_PAGES: u32 = 65536; // 4GB in 64KB pages

    if config.max_memory_pages == 0 || config.max_memory_pages > MAX_MEMORY_PAGES {
        return Err(ConfigValidationError::InvalidMemoryPages(
            config.max_memory_pages,
            MAX_MEMORY_PAGES,
        ));
    }

    const MAX_TABLE_SIZE: u32 = 10_000_000;

    if config.max_table_size > MAX_TABLE_SIZE {
        return Err(ConfigValidationError::InvalidTableSize(
            config.max_table_size,
            MAX_TABLE_SIZE,
        ));
    }

    const MIN_STACK_SIZE: u32 = 16;

    if config.stack_size_limit < MIN_STACK_SIZE {
        return Err(ConfigValidationError::InvalidStackSize(
            config.stack_size_limit,
            MIN_STACK_SIZE,
        ));
    }

    // Check for inconsistent feature flags
    if config.enable_multi_value && !config.strict_validation {
        return Err(ConfigValidationError::InconsistentFeatures(
            "Multi-value requires strict validation".to_string(),
        ));
    }

    Ok(())
}

/// Validate output configuration
pub fn validate_output(config: &super::options::OutputConfig) -> ValidationResult<()> {
    // Check that output paths are valid if specified
    if let Some(ref nef_path) = config.nef_path {
        if nef_path.as_os_str().is_empty() {
            return Err(ConfigValidationError::InvalidOutput(
                "NEF path cannot be empty".to_string(),
            ));
        }
    }

    if let Some(ref manifest_path) = config.manifest_path {
        if manifest_path.as_os_str().is_empty() {
            return Err(ConfigValidationError::InvalidOutput(
                "Manifest path cannot be empty".to_string(),
            ));
        }
    }

    Ok(())
}

/// Get a summary of the configuration for logging
pub fn config_summary(config: &TranslationConfig) -> String {
    format!(
        "TranslationConfig {{ \
        contract: {}, \
        source_chain: {:?}, \
        max_memory: {} pages, \
        features: [bulk_memory={}, ref_types={}, multi_value={}] \
        }}",
        config.contract_name,
        config.source_chain,
        config.behavior.max_memory_pages,
        config.behavior.enable_bulk_memory,
        config.behavior.enable_reference_types,
        config.behavior.enable_multi_value
    )
}

/// Sanitize a configuration by applying safe defaults
pub fn sanitize_config(mut config: TranslationConfig) -> TranslationConfig {
    // Ensure memory pages is at least 1
    if config.behavior.max_memory_pages == 0 {
        config.behavior.max_memory_pages = 1;
    }

    // Cap memory pages to reasonable limit
    const REASONABLE_MAX_PAGES: u32 = 16384; // 1GB
    if config.behavior.max_memory_pages > REASONABLE_MAX_PAGES {
        config.behavior.max_memory_pages = REASONABLE_MAX_PAGES;
    }

    // Ensure stack size is reasonable
    #[allow(clippy::manual_clamp)]
    {
        if config.behavior.stack_size_limit < 16 {
            config.behavior.stack_size_limit = 16;
        }
        if config.behavior.stack_size_limit > 65536 {
            config.behavior.stack_size_limit = 65536;
        }
    }

    config
}

/// Check if a log level should be shown given the current configuration
pub fn should_log(config_level: LogLevel, message_level: LogLevel) -> bool {
    message_level <= config_level
}

/// Get the appropriate log level for a given verbosity setting
pub fn verbosity_to_log_level(verbose: bool, very_verbose: bool) -> LogLevel {
    if very_verbose {
        LogLevel::Trace
    } else if verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

#[cfg(test)]
mod tests {
    use super::super::options::*;
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = TranslationConfig::new("TestContract");
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_memory_pages() {
        let config = TranslationConfig::new("Test")
            .with_behavior(BehaviorConfig::default().with_max_memory_pages(0));
        assert!(matches!(
            validate_config(&config),
            Err(ConfigValidationError::InvalidMemoryPages(0, _))
        ));

        let config = TranslationConfig::new("Test")
            .with_behavior(BehaviorConfig::default().with_max_memory_pages(100000));
        assert!(matches!(
            validate_config(&config),
            Err(ConfigValidationError::InvalidMemoryPages(100000, _))
        ));
    }

    #[test]
    fn test_sanitize_config() {
        let config = TranslationConfig::new("Test")
            .with_behavior(BehaviorConfig::default().with_max_memory_pages(0));

        let sanitized = sanitize_config(config);
        assert_eq!(sanitized.behavior.max_memory_pages, 1);

        let config = TranslationConfig::new("Test")
            .with_behavior(BehaviorConfig::default().with_max_memory_pages(100000));

        let sanitized = sanitize_config(config);
        assert_eq!(sanitized.behavior.max_memory_pages, 16384);
    }

    #[test]
    fn test_should_log() {
        assert!(should_log(LogLevel::Debug, LogLevel::Error));
        assert!(should_log(LogLevel::Debug, LogLevel::Debug));
        assert!(!should_log(LogLevel::Info, LogLevel::Debug));
    }

    #[test]
    fn test_verbosity_to_log_level() {
        assert_eq!(verbosity_to_log_level(false, false), LogLevel::Info);
        assert_eq!(verbosity_to_log_level(true, false), LogLevel::Debug);
        assert_eq!(verbosity_to_log_level(true, true), LogLevel::Trace);
    }
}
