// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Configuration options for the WASM to NeoVM translator

use std::path::PathBuf;

use serde_json::Value;

use crate::adapters::SourceChain;
use crate::logging::LogLevel;
use crate::types::ContractName;

/// Primary configuration structure for WASM to NeoVM translation
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    /// Contract name for the output
    pub contract_name: ContractName,

    /// Source blockchain for cross-chain compilation
    pub source_chain: SourceChain,

    /// Source URL for NEF metadata
    pub source_url: Option<String>,

    /// Manifest overlay file path
    pub manifest_overlay: Option<PathBuf>,

    /// Reference manifest for comparison
    pub compare_manifest: Option<PathBuf>,

    /// Output paths
    pub output: OutputConfig,

    /// Translation behavior options
    pub behavior: BehaviorConfig,

    /// Debugging and profiling options
    pub debug: DebugConfig,

    /// Extra manifest overlay data
    pub extra_manifest_overlay: Option<ManifestOverlay>,
}

/// Manifest overlay data for customizing contract manifest
#[derive(Debug, Clone)]
pub struct ManifestOverlay {
    /// The overlay JSON value
    pub value: Value,
    /// Optional label for debugging
    pub label: Option<String>,
}

impl TranslationConfig {
    /// Create a new translation config with the given contract name.
    ///
    /// Empty names are normalized to `"Contract"` to avoid constructor-time panics.
    pub fn new(contract_name: impl AsRef<str>) -> Self {
        let contract_name = ContractName::try_new(contract_name.as_ref())
            .unwrap_or_else(|| ContractName::new("Contract"));

        Self {
            contract_name,
            source_chain: SourceChain::default(),
            source_url: None,
            manifest_overlay: None,
            compare_manifest: None,
            output: OutputConfig::default(),
            behavior: BehaviorConfig::default(),
            debug: DebugConfig::default(),
            extra_manifest_overlay: None,
        }
    }

    /// Set the source chain
    pub fn with_source_chain(mut self, chain: SourceChain) -> Self {
        self.source_chain = chain;
        self
    }

    /// Set the source URL
    pub fn with_source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    /// Set the manifest overlay file path
    pub fn with_manifest_overlay_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_overlay = Some(path.into());
        self
    }

    /// Set the compare manifest path
    pub fn with_compare_manifest(mut self, path: impl Into<PathBuf>) -> Self {
        self.compare_manifest = Some(path.into());
        self
    }

    /// Set output configuration
    pub fn with_output(mut self, output: OutputConfig) -> Self {
        self.output = output;
        self
    }

    /// Set behavior configuration
    pub fn with_behavior(mut self, behavior: BehaviorConfig) -> Self {
        self.behavior = behavior;
        self
    }

    /// Set debug configuration
    pub fn with_debug(mut self, debug: DebugConfig) -> Self {
        self.debug = debug;
        self
    }

    /// Set extra manifest overlay
    pub fn with_manifest_overlay(mut self, overlay: ManifestOverlay) -> Self {
        self.extra_manifest_overlay = Some(overlay);
        self
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self::new("Contract")
    }
}

/// Output path configuration
#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    /// Output NEF file path
    pub nef_path: Option<PathBuf>,

    /// Output manifest file path
    pub manifest_path: Option<PathBuf>,

    /// Output directory (default: same as input)
    pub output_dir: Option<PathBuf>,

    /// Whether to write intermediate files
    pub write_intermediates: bool,

    /// Intermediate output directory
    pub intermediate_dir: Option<PathBuf>,
}

impl OutputConfig {
    /// Set the NEF output path
    pub fn with_nef_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.nef_path = Some(path.into());
        self
    }

    /// Set the manifest output path
    pub fn with_manifest_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_path = Some(path.into());
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(dir.into());
        self
    }

    /// Enable intermediate file output
    pub fn with_intermediates(mut self, dir: impl Into<PathBuf>) -> Self {
        self.write_intermediates = true;
        self.intermediate_dir = Some(dir.into());
        self
    }
}

/// Translation behavior configuration
#[derive(Debug, Clone)]
pub struct BehaviorConfig {
    /// Maximum allowed memory size (in pages)
    pub max_memory_pages: u32,

    /// Maximum allowed table size
    pub max_table_size: u32,

    /// Whether to enable bulk memory operations
    pub enable_bulk_memory: bool,

    /// Whether to enable reference types
    pub enable_reference_types: bool,

    /// Whether to enable multi-value returns
    pub enable_multi_value: bool,

    /// Stack size limit for the generated code
    pub stack_size_limit: u32,

    /// Whether to perform aggressive optimizations
    pub aggressive_optimization: bool,

    /// Whether to validate WASM strictly
    pub strict_validation: bool,

    /// Whether to allow floating point operations
    pub allow_float: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 1024, // 64MB default limit
            max_table_size: 10000,
            enable_bulk_memory: true,
            enable_reference_types: true,
            enable_multi_value: false, // Not fully supported yet
            stack_size_limit: 1024,
            aggressive_optimization: false,
            strict_validation: true,
            allow_float: false,
        }
    }
}

impl BehaviorConfig {
    /// Set maximum memory pages
    pub fn with_max_memory_pages(mut self, pages: u32) -> Self {
        self.max_memory_pages = pages;
        self
    }

    /// Set maximum table size
    pub fn with_max_table_size(mut self, size: u32) -> Self {
        self.max_table_size = size;
        self
    }

    /// Enable bulk memory operations
    pub fn with_bulk_memory(mut self, enable: bool) -> Self {
        self.enable_bulk_memory = enable;
        self
    }

    /// Enable reference types
    pub fn with_reference_types(mut self, enable: bool) -> Self {
        self.enable_reference_types = enable;
        self
    }

    /// Enable multi-value returns
    pub fn with_multi_value(mut self, enable: bool) -> Self {
        self.enable_multi_value = enable;
        self
    }

    /// Set stack size limit
    pub fn with_stack_size_limit(mut self, limit: u32) -> Self {
        self.stack_size_limit = limit;
        self
    }

    /// Enable aggressive optimization
    pub fn with_aggressive_optimization(mut self, enable: bool) -> Self {
        self.aggressive_optimization = enable;
        self
    }

    /// Set strict validation
    pub fn with_strict_validation(mut self, strict: bool) -> Self {
        self.strict_validation = strict;
        self
    }

    /// Allow floating point operations
    pub fn with_float(mut self, allow: bool) -> Self {
        self.allow_float = allow;
        self
    }
}

/// Debug and profiling configuration
#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    /// Enable detailed logging
    pub verbose: bool,

    /// Enable profiling instrumentation
    pub enable_profiling: bool,

    /// Dump intermediate representations
    pub dump_ir: bool,

    /// Dump WASM disassembly
    pub dump_wasm: bool,

    /// Dump generated bytecode
    pub dump_bytecode: bool,

    /// Dump manifest before/after processing
    pub dump_manifest: bool,

    /// Log level override
    pub log_level: Option<LogLevel>,
}

impl DebugConfig {
    /// Enable verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable profiling
    pub fn with_profiling(mut self, enable: bool) -> Self {
        self.enable_profiling = enable;
        self
    }

    /// Enable IR dumping
    pub fn with_dump_ir(mut self, dump: bool) -> Self {
        self.dump_ir = dump;
        self
    }

    /// Enable WASM dumping
    pub fn with_dump_wasm(mut self, dump: bool) -> Self {
        self.dump_wasm = dump;
        self
    }

    /// Enable bytecode dumping
    pub fn with_dump_bytecode(mut self, dump: bool) -> Self {
        self.dump_bytecode = dump;
        self
    }

    /// Enable manifest dumping
    pub fn with_dump_manifest(mut self, dump: bool) -> Self {
        self.dump_manifest = dump;
        self
    }

    /// Set log level
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_config_builder() {
        let config = TranslationConfig::new("TestContract")
            .with_source_chain(SourceChain::Solana)
            .with_source_url("https://example.com")
            .with_behavior(BehaviorConfig::default().with_max_memory_pages(2048));

        assert_eq!(config.contract_name.as_str(), "TestContract");
        assert_eq!(config.source_chain, SourceChain::Solana);
        assert_eq!(config.source_url, Some("https://example.com".to_string()));
        assert_eq!(config.behavior.max_memory_pages, 2048);
    }

    #[test]
    fn test_translation_config_empty_name_defaults_to_contract() {
        let config = TranslationConfig::new("");
        assert_eq!(config.contract_name.as_str(), "Contract");
    }

    #[test]
    fn test_behavior_config() {
        let config = BehaviorConfig::default()
            .with_bulk_memory(true)
            .with_aggressive_optimization(true);

        assert!(config.enable_bulk_memory);
        assert!(config.aggressive_optimization);
    }

    #[test]
    fn test_debug_config() {
        let config = DebugConfig::default()
            .with_verbose(true)
            .with_profiling(true)
            .with_log_level(LogLevel::Debug);

        assert!(config.verbose);
        assert!(config.enable_profiling);
        assert_eq!(config.log_level, Some(LogLevel::Debug));
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }
}
