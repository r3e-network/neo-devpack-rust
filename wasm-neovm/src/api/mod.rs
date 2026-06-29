// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Fluent / convenience translation API.
//!
//! [`TranslationBuilder`] provides a builder-style entry point and
//! [`translate_wasm`] a one-call helper; both wrap the crate-root
//! `translate_module` / `translate_with_config`. The core configuration and
//! result types they consume and return are re-exported from the crate root
//! (`wasm_neovm::*`), not duplicated here.

use crate::adapters::SourceChain;
use crate::config::{BehaviorConfig, DebugConfig, OutputConfig, TranslationConfig};
use crate::translator::{translate_module, translate_with_config, Translation};
use crate::types::ContractName;

/// Statistics about a translation
#[derive(Debug, Clone, Default)]
pub struct TranslationStats {
    /// Number of exported methods in the ABI
    pub export_count: usize,
    /// Size of the generated script in bytes
    pub script_size: usize,
    /// Number of method tokens
    pub token_count: usize,
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
            export_count: method_count,
            script_size: translation.script.len(),
            token_count: translation.method_tokens.len(),
            translation_time_ms: None,
        }
    }

    /// Set the translation time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.translation_time_ms = Some(ms);
        self
    }
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
    fn test_translation_stats() {
        let stats = TranslationStats {
            export_count: 5,
            script_size: 1024,
            token_count: 3,
            translation_time_ms: Some(100),
        };

        assert_eq!(stats.export_count, 5);
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
