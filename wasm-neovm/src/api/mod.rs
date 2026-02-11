// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! API consistency layer
//!
//! This module ensures consistent naming across public APIs
//! and provides deprecation aliases for backward compatibility.

pub use crate::adapters::SourceChain;
pub use crate::config::{BehaviorConfig, DebugConfig, OutputConfig, TranslationConfig};
pub use crate::logging::{LogCategory, LogLevel};
pub use crate::manifest::{ManifestMethod, ManifestParameter, RenderedManifest};
pub use crate::metadata::NefMetadata;
pub use crate::nef::{write_nef, write_nef_with_metadata, MethodToken};
pub use crate::translator::{
    translate_module, translate_with_config, ManifestOverlay, Translation,
};
pub use crate::types::{
    BytecodeOffset, ContractName, GlobalIndex, LocalIndex, MemoryOffset, MethodIndex,
    SyscallDescriptor, WasmValueType,
};

/// Result type for translation operations
pub type TranslationResult<T> = anyhow::Result<T>;

/// Error type for translation operations
pub type TranslationError = anyhow::Error;

/// Version of the wasm-neovm library
pub const LIB_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported WASM features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WasmFeatures {
    /// Bulk memory operations (memory.copy, memory.fill, etc.)
    pub bulk_memory: bool,
    /// Reference types (funcref, externref)
    pub reference_types: bool,
    /// Multi-value returns
    pub multi_value: bool,
    /// Mutable globals
    pub mutable_globals: bool,
    /// Sign-extension operations
    pub sign_extension: bool,
    /// Non-trapping float-to-int conversion
    pub sat_float_to_int: bool,
}

impl Default for WasmFeatures {
    fn default() -> Self {
        Self {
            bulk_memory: true,
            reference_types: true,
            multi_value: false,
            mutable_globals: true,
            sign_extension: true,
            sat_float_to_int: false,
        }
    }
}

impl WasmFeatures {
    /// Create with all features enabled
    pub fn all() -> Self {
        Self {
            bulk_memory: true,
            reference_types: true,
            multi_value: true,
            mutable_globals: true,
            sign_extension: true,
            sat_float_to_int: true,
        }
    }

    /// Create with all features disabled
    pub fn none() -> Self {
        Self {
            bulk_memory: false,
            reference_types: false,
            multi_value: false,
            mutable_globals: false,
            sign_extension: false,
            sat_float_to_int: false,
        }
    }

    /// Enable all features
    pub fn enable_all(&mut self) -> &mut Self {
        *self = Self::all();
        self
    }

    /// Disable all features
    pub fn disable_all(&mut self) -> &mut Self {
        *self = Self::none();
        self
    }

    /// Check if any feature is enabled
    pub fn has_any(&self) -> bool {
        self.bulk_memory
            || self.reference_types
            || self.multi_value
            || self.mutable_globals
            || self.sign_extension
            || self.sat_float_to_int
    }
}

/// Statistics about a translation
#[derive(Debug, Clone, Default)]
pub struct TranslationStats {
    /// Number of functions translated
    pub function_count: usize,
    /// Size of the generated script in bytes
    pub script_size: usize,
    /// Number of method tokens
    pub token_count: usize,
    /// Number of exported functions
    pub export_count: usize,
    /// Number of imports
    pub import_count: usize,
    /// Translation time in milliseconds (if measured)
    pub translation_time_ms: Option<u64>,
}

impl TranslationStats {
    /// Create new stats from a translation result
    pub fn from_translation(translation: &Translation) -> Self {
        // Extract method count from manifest JSON
        let method_count = translation
            .manifest
            .value
            .get("abi")
            .and_then(|abi| abi.get("methods"))
            .and_then(|m| m.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Self {
            function_count: method_count,
            script_size: translation.script.len(),
            token_count: translation.method_tokens.len(),
            export_count: method_count,
            import_count: 0, // Would need to track this during translation
            translation_time_ms: None,
        }
    }

    /// Set the translation time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.translation_time_ms = Some(ms);
        self
    }
}

/// Information about a translated contract
#[derive(Debug, Clone)]
pub struct ContractInfo {
    /// Contract name
    pub name: ContractName,
    /// Contract version (if specified)
    pub version: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Supported standards
    pub standards: Vec<String>,
    /// Contract permissions
    pub permissions: Vec<String>,
    /// Trust level
    pub trusts: Option<String>,
}

/// Builder for fluent API to configure and run translation
#[derive(Debug)]
pub struct TranslationBuilder {
    config: TranslationConfig,
    wasm_bytes: Option<Vec<u8>>,
}

impl TranslationBuilder {
    /// Create a new translation builder.
    ///
    /// Empty names are normalized to `"Contract"` to avoid constructor-time panics.
    pub fn new(contract_name: impl AsRef<str>) -> Self {
        Self {
            config: TranslationConfig::new(contract_name),
            wasm_bytes: None,
        }
    }

    /// Create a new translation builder with explicit contract-name validation.
    pub fn try_new(contract_name: impl AsRef<str>) -> anyhow::Result<Self> {
        let contract_name = contract_name.as_ref();
        let contract_name = ContractName::try_new(contract_name)
            .ok_or_else(|| anyhow::anyhow!("contract name cannot be empty"))?;
        Ok(Self {
            config: TranslationConfig::new(contract_name),
            wasm_bytes: None,
        })
    }

    /// Set the WASM input bytes
    pub fn with_wasm(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.wasm_bytes = Some(bytes.into());
        self
    }

    /// Set the WASM input from a file
    pub fn with_wasm_file(self, path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)?;
        Ok(self.with_wasm(bytes))
    }

    /// Set the source chain
    pub fn from_chain(mut self, chain: SourceChain) -> Self {
        self.config.source_chain = chain;
        self
    }

    /// Set behavior options
    pub fn with_behavior(mut self, behavior: BehaviorConfig) -> Self {
        self.config.behavior = behavior;
        self
    }

    /// Set output options
    pub fn with_output(mut self, output: OutputConfig) -> Self {
        self.config.output = output;
        self
    }

    /// Set debug options
    pub fn with_debug(mut self, debug: DebugConfig) -> Self {
        self.config.debug = debug;
        self
    }

    /// Set source URL
    pub fn with_source_url(mut self, url: impl Into<String>) -> Self {
        self.config.source_url = Some(url.into());
        self
    }

    /// Execute the translation
    pub fn translate(self) -> anyhow::Result<Translation> {
        let bytes = self.wasm_bytes.ok_or_else(|| {
            anyhow::anyhow!("WASM bytes not set. Use with_wasm() or with_wasm_file()")
        })?;
        translate_with_config(&bytes, self.config)
    }

    /// Execute the translation and return with stats
    pub fn translate_with_stats(self) -> anyhow::Result<(Translation, TranslationStats)> {
        let start = std::time::Instant::now();
        let translation = self.translate()?;
        let elapsed = start.elapsed().as_millis() as u64;

        let mut stats = TranslationStats::from_translation(&translation);
        stats.translation_time_ms = Some(elapsed);

        Ok((translation, stats))
    }
}

/// Convenience function for quick translation
///
/// # Example
/// ```rust,ignore
/// let translation = translate_wasm(&wasm_bytes, "MyContract")?;
/// ```
pub fn translate_wasm(
    wasm_bytes: &[u8],
    contract_name: impl AsRef<str>,
) -> anyhow::Result<Translation> {
    let contract_name = contract_name.as_ref();
    let _ = ContractName::try_new(contract_name)
        .ok_or_else(|| anyhow::anyhow!("contract name cannot be empty"))?;
    translate_module(wasm_bytes, contract_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_features() {
        let features = WasmFeatures::default();
        assert!(features.bulk_memory);
        assert!(features.reference_types);
        assert!(!features.multi_value);

        let all = WasmFeatures::all();
        assert!(all.has_any());

        let none = WasmFeatures::none();
        assert!(!none.has_any());
    }

    #[test]
    fn test_translation_stats() {
        let stats = TranslationStats {
            function_count: 5,
            script_size: 1024,
            token_count: 3,
            export_count: 5,
            import_count: 2,
            translation_time_ms: Some(100),
        };

        assert_eq!(stats.function_count, 5);
        assert_eq!(stats.script_size, 1024);
    }

    #[test]
    fn test_translation_builder() {
        let builder = TranslationBuilder::new("TestContract")
            .from_chain(SourceChain::Neo)
            .with_wasm(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

        assert!(builder.wasm_bytes.is_some());
    }

    #[test]
    fn test_translation_builder_new_empty_name_defaults_to_contract() {
        let builder = TranslationBuilder::new("")
            .from_chain(SourceChain::Neo)
            .with_wasm(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

        assert_eq!(builder.config.contract_name.as_str(), "Contract");
    }

    #[test]
    fn test_translation_builder_try_new_rejects_empty_contract_name() {
        let err = TranslationBuilder::try_new("").expect_err("empty contract name should error");
        assert!(err
            .to_string()
            .to_ascii_lowercase()
            .contains("contract name cannot be empty"));
    }

    #[test]
    fn test_translate_wasm_accepts_contract_name_value() {
        let wasm = wat::parse_str(
            r#"(module
                  (func (export "main")
                    nop)
                )"#,
        )
        .expect("valid wat");

        let name = ContractName::new("TypedName");
        let translation = translate_wasm(&wasm, &name).expect("translation should succeed");
        assert_eq!(translation.contract_name.as_str(), "TypedName");
    }

    #[test]
    fn test_translate_wasm_rejects_empty_contract_name() {
        let wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let err = translate_wasm(&wasm, "").expect_err("empty contract name should error");
        assert!(err
            .to_string()
            .to_ascii_lowercase()
            .contains("contract name cannot be empty"));
    }
}
