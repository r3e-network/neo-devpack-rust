// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use crate::manifest::{ManifestMethod, RenderedManifest};
use crate::nef::MethodToken;
use crate::types::ContractName;

/// Optimized stack value representation (Round 84 - Cache Locality)
///
/// Layout optimized for cache efficiency:
/// - const_value: 24 bytes (`Option<i128>` with tag)
/// - bytecode_start: 16 bytes (`Option<usize>` with tag)
///
/// Total: 40 bytes (padded to 40 on 64-bit)
///
/// Previous layout had worse cache behavior due to field ordering.
#[derive(Debug, Clone)]
pub(crate) struct StackValue {
    /// Constant value if known at compile time
    pub(crate) const_value: Option<i128>,
    /// Bytecode offset where this value starts (for backtracking optimizations)
    pub(crate) bytecode_start: Option<usize>,
}

impl StackValue {
    /// Create a new stack value with unknown constant (Round 81 - inline hot constructor)
    #[inline(always)]
    pub(crate) fn unknown() -> Self {
        Self {
            const_value: None,
            bytecode_start: None,
        }
    }

    /// Create a new stack value with known constant (Round 81 - inline hot constructor)
    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn constant(value: i128) -> Self {
        Self {
            const_value: Some(value),
            bytecode_start: None,
        }
    }
}

/// The result of translating a WASM module to NeoVM
#[derive(Debug)]
pub struct Translation {
    /// The generated NeoVM bytecode script
    pub script: Vec<u8>,
    /// Method tokens for cross-contract calls
    pub method_tokens: Vec<MethodToken>,
    /// The generated contract manifest
    pub manifest: RenderedManifest,
    /// Source URL for the contract (if provided)
    pub source_url: Option<String>,
    /// Contract name
    pub contract_name: ContractName,
}

impl Translation {
    /// Create a new translation result
    pub fn new(
        script: Vec<u8>,
        method_tokens: Vec<MethodToken>,
        manifest: RenderedManifest,
        contract_name: ContractName,
    ) -> Self {
        Self {
            script,
            method_tokens,
            manifest,
            source_url: None,
            contract_name,
        }
    }

    /// Set the source URL
    pub fn with_source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    /// Get the script size in bytes
    pub fn script_size(&self) -> usize {
        self.script.len()
    }

    /// Get the total method token count
    pub fn token_count(&self) -> usize {
        self.method_tokens.len()
    }
}

#[derive(Debug)]
/// Intermediate manifest data collected during translation.
pub struct ManifestData {
    /// ABI methods extracted from WASM exports.
    pub methods: Vec<ManifestMethod>,
}

// Re-export the centralized config for backward compatibility
pub use crate::config::options::{ManifestOverlay, TranslationConfig};
