use crate::adapters::SourceChain;
use crate::manifest::{ManifestMethod, RenderedManifest};
use crate::nef::MethodToken;
use serde_json::Value;

/// Optimized stack value representation (Round 84 - Cache Locality)
///
/// Layout optimized for cache efficiency:
/// - const_value: 24 bytes (Option<i128> with tag)
/// - bytecode_start: 16 bytes (Option<usize> with tag)
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

#[derive(Debug)]
pub struct Translation {
    pub script: Vec<u8>,
    pub method_tokens: Vec<MethodToken>,
    pub manifest: RenderedManifest,
    pub source_url: Option<String>,
}

#[derive(Debug)]
pub struct ManifestData {
    pub methods: Vec<ManifestMethod>,
}

#[derive(Debug)]
pub struct TranslationConfig<'a> {
    pub contract_name: &'a str,
    pub extra_manifest_overlay: Option<ManifestOverlay>,
    pub source_chain: SourceChain,
}

impl<'a> TranslationConfig<'a> {
    pub fn new(contract_name: &'a str) -> Self {
        Self {
            contract_name,
            extra_manifest_overlay: None,
            source_chain: SourceChain::Neo,
        }
    }

    pub fn with_manifest_overlay(mut self, overlay: ManifestOverlay) -> Self {
        self.extra_manifest_overlay = Some(overlay);
        self
    }

    pub fn with_source_chain(mut self, source_chain: SourceChain) -> Self {
        self.source_chain = source_chain;
        self
    }
}

#[derive(Debug)]
pub struct ManifestOverlay {
    pub value: Value,
    pub label: Option<String>,
}
