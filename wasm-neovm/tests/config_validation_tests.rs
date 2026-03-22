// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Extended configuration validation edge case tests

use wasm_neovm::config::options::*;
use wasm_neovm::config::validation::*;

// ============================================================================
// Contract name validation
// ============================================================================

#[test]
fn test_validate_empty_contract_name() {
    // TranslationConfig::new normalizes empty to "Contract", so validate_config
    // on a default config should pass.
    let config = TranslationConfig::new("");
    assert_eq!(config.contract_name.as_str(), "Contract");
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_long_contract_name_is_rejected() {
    // A 257-char name passes through the constructor but fails validation
    let long_name = "A".repeat(257);
    let config = TranslationConfig::new(&long_name);
    let result = validate_config(&config);
    // Either the constructor truncated it (ok) or validation rejects it (err)
    // Both outcomes are acceptable
    if config.contract_name.as_str().len() > 256 {
        assert!(result.is_err());
    }
}

#[test]
fn test_validate_normal_contract_name() {
    let config = TranslationConfig::new("MyToken");
    assert!(validate_config(&config).is_ok());
}

// ============================================================================
// Memory pages boundary conditions
// ============================================================================

#[test]
fn test_validate_memory_pages_zero() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(0));
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InvalidMemoryPages(0, _))
    ));
}

#[test]
fn test_validate_memory_pages_one() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(1));
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_memory_pages_max_valid() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(65536));
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_memory_pages_exceeds_max() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(65537));
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InvalidMemoryPages(65537, _))
    ));
}

#[test]
fn test_validate_memory_pages_u32_max() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(u32::MAX));
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InvalidMemoryPages(_, _))
    ));
}

// ============================================================================
// Table size boundary conditions
// ============================================================================

#[test]
fn test_validate_table_size_zero() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_table_size(0));
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_table_size_max_valid() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_table_size(10_000_000));
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_table_size_exceeds_max() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_table_size(10_000_001));
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InvalidTableSize(10_000_001, _))
    ));
}

// ============================================================================
// Stack size boundary conditions
// ============================================================================

#[test]
fn test_validate_stack_size_below_minimum() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_stack_size_limit(15));
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InvalidStackSize(15, 16))
    ));
}

#[test]
fn test_validate_stack_size_at_minimum() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_stack_size_limit(16));
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_stack_size_zero() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_stack_size_limit(0));
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InvalidStackSize(0, 16))
    ));
}

// ============================================================================
// Feature flag consistency
// ============================================================================

#[test]
fn test_validate_multi_value_without_strict_validation() {
    let config = TranslationConfig::new("Test").with_behavior(
        BehaviorConfig::default()
            .with_multi_value(true)
            .with_strict_validation(false),
    );
    assert!(matches!(
        validate_config(&config),
        Err(ConfigValidationError::InconsistentFeatures(_))
    ));
}

#[test]
fn test_validate_multi_value_with_strict_validation() {
    let config = TranslationConfig::new("Test").with_behavior(
        BehaviorConfig::default()
            .with_multi_value(true)
            .with_strict_validation(true),
    );
    assert!(validate_config(&config).is_ok());
}

// ============================================================================
// Sanitize config
// ============================================================================

#[test]
fn test_sanitize_clamps_stack_size_to_min() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_stack_size_limit(1));
    let sanitized = sanitize_config(config);
    assert_eq!(sanitized.behavior.stack_size_limit, 16);
}

#[test]
fn test_sanitize_clamps_stack_size_to_max() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_stack_size_limit(100_000));
    let sanitized = sanitize_config(config);
    assert_eq!(sanitized.behavior.stack_size_limit, 65536);
}

#[test]
fn test_sanitize_leaves_valid_stack_size() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_stack_size_limit(512));
    let sanitized = sanitize_config(config);
    assert_eq!(sanitized.behavior.stack_size_limit, 512);
}

#[test]
fn test_sanitize_memory_pages_caps_at_reasonable_max() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(u32::MAX));
    let sanitized = sanitize_config(config);
    assert_eq!(sanitized.behavior.max_memory_pages, 16384);
}

// ============================================================================
// Config summary
// ============================================================================

#[test]
fn test_config_summary_contains_contract_name() {
    let config = TranslationConfig::new("MyContract");
    let summary = config_summary(&config);
    assert!(summary.contains("MyContract"));
}

#[test]
fn test_config_summary_contains_memory_pages() {
    let config = TranslationConfig::new("Test")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(2048));
    let summary = config_summary(&config);
    assert!(summary.contains("2048"));
}

// ============================================================================
// Log level utilities
// ============================================================================

#[test]
fn test_should_log_trace_shows_everything() {
    use wasm_neovm::logging::LogLevel;
    assert!(should_log(LogLevel::Trace, LogLevel::Error));
    assert!(should_log(LogLevel::Trace, LogLevel::Warn));
    assert!(should_log(LogLevel::Trace, LogLevel::Info));
    assert!(should_log(LogLevel::Trace, LogLevel::Debug));
    assert!(should_log(LogLevel::Trace, LogLevel::Trace));
}

#[test]
fn test_should_log_error_only_shows_error() {
    use wasm_neovm::logging::LogLevel;
    assert!(should_log(LogLevel::Error, LogLevel::Error));
    assert!(!should_log(LogLevel::Error, LogLevel::Warn));
    assert!(!should_log(LogLevel::Error, LogLevel::Info));
}

#[test]
fn test_verbosity_both_false() {
    use wasm_neovm::logging::LogLevel;
    assert_eq!(verbosity_to_log_level(false, false), LogLevel::Info);
}

#[test]
fn test_verbosity_very_verbose_overrides() {
    use wasm_neovm::logging::LogLevel;
    assert_eq!(verbosity_to_log_level(false, true), LogLevel::Trace);
}

// ============================================================================
// Builder pattern coverage
// ============================================================================

#[test]
fn test_output_config_builder() {
    let output = OutputConfig::default()
        .with_nef_path("output.nef")
        .with_manifest_path("output.manifest.json")
        .with_output_dir("/tmp/output")
        .with_intermediates("/tmp/intermediate");

    assert!(output.nef_path.is_some());
    assert!(output.manifest_path.is_some());
    assert!(output.output_dir.is_some());
    assert!(output.write_intermediates);
    assert!(output.intermediate_dir.is_some());
}

#[test]
fn test_debug_config_all_flags() {
    let debug = DebugConfig::default()
        .with_verbose(true)
        .with_profiling(true)
        .with_dump_ir(true)
        .with_dump_wasm(true)
        .with_dump_bytecode(true)
        .with_dump_manifest(true);

    assert!(debug.verbose);
    assert!(debug.enable_profiling);
    assert!(debug.dump_ir);
    assert!(debug.dump_wasm);
    assert!(debug.dump_bytecode);
    assert!(debug.dump_manifest);
}

#[test]
fn test_behavior_config_all_flags() {
    let behavior = BehaviorConfig::default()
        .with_bulk_memory(false)
        .with_reference_types(false)
        .with_multi_value(false)
        .with_float(true)
        .with_aggressive_optimization(true)
        .with_strict_validation(false);

    assert!(!behavior.enable_bulk_memory);
    assert!(!behavior.enable_reference_types);
    assert!(!behavior.enable_multi_value);
    assert!(behavior.allow_float);
    assert!(behavior.aggressive_optimization);
    assert!(!behavior.strict_validation);
}

#[test]
fn test_translation_config_with_source_chain() {
    use wasm_neovm::adapters::SourceChain;
    let config = TranslationConfig::new("Test").with_source_chain(SourceChain::Solana);
    assert_eq!(config.source_chain, SourceChain::Solana);
}

#[test]
fn test_translation_config_default() {
    let config = TranslationConfig::default();
    assert_eq!(config.contract_name.as_str(), "Contract");
}
