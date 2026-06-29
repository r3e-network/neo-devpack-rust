// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::reachability::{
    analyze_function_dependency_graph, analyze_reachable_defined_functions, FunctionDependencyGraph,
};
use super::*;
use crate::config::{BehaviorConfig, ManifestOverlay};

use exports::ExportedFunction;

pub(super) struct DriverState {
    pub(super) contract_name: String,
    pub(super) adapter: Box<dyn ChainAdapter>,
    pub(super) extra_manifest_overlay: Option<ManifestOverlay>,
    pub(super) behavior: BehaviorConfig,
    pub(super) frontend: ModuleFrontend,
    pub(super) exported_funcs: BTreeMap<u32, ExportedFunction>,
    pub(super) import_export_indices: BTreeSet<usize>,
    pub(super) tables: Vec<TableInfo>,
    pub(super) script: Vec<u8>,
    pub(super) runtime: RuntimeHelpers,
    pub(super) feature_tracker: FeatureTracker,
    pub(super) methods: Vec<ManifestMethod>,
    pub(super) overlay_safe_methods: HashSet<String>,
    pub(super) manifest_overlay: Option<Value>,
    pub(super) section_method_tokens: Vec<MethodToken>,
    pub(super) section_source: Option<String>,
    pub(super) saw_code_section: bool,
    pub(super) next_defined_index: usize,
    pub(super) function_registry: Option<FunctionRegistry>,
    pub(super) start_function: Option<u32>,
    pub(super) start_defined_offset: Option<usize>,
    pub(super) reachable_defined_functions: Option<HashSet<u32>>,
    pub(super) function_dependency_graph: Option<FunctionDependencyGraph>,
    pub(super) direct_runtime_init_functions: HashSet<u32>,
    pub(super) method_function_indices: BTreeMap<String, u32>,
}

impl DriverState {
    pub(super) fn new(config: TranslationConfig) -> Self {
        // Pre-size collections based on typical contract sizes (Round 62, 63 optimizations)
        const TYPICAL_SCRIPT_CAPACITY: usize = 4096;
        const TYPICAL_METHODS_CAPACITY: usize = 32;

        Self {
            contract_name: config.contract_name.into_inner(),
            adapter: get_adapter(config.source_chain),
            extra_manifest_overlay: config.extra_manifest_overlay,
            behavior: config.behavior,
            frontend: ModuleFrontend::new(),
            exported_funcs: BTreeMap::new(),
            import_export_indices: BTreeSet::new(),
            tables: Vec::with_capacity(4),
            // Pre-allocate script buffer with typical capacity (Round 62 optimization)
            script: Vec::with_capacity(TYPICAL_SCRIPT_CAPACITY),
            // Use pre-allocated RuntimeHelpers (Round 62, 63 optimizations)
            runtime: RuntimeHelpers::with_capacity(8, 8, 16),
            feature_tracker: FeatureTracker::default(),
            methods: Vec::with_capacity(TYPICAL_METHODS_CAPACITY),
            // Pre-size HashSet for faster lookups (Round 63 optimization)
            overlay_safe_methods: HashSet::with_capacity(8),
            manifest_overlay: None,
            section_method_tokens: Vec::with_capacity(16),
            section_source: None,
            saw_code_section: false,
            next_defined_index: 0,
            function_registry: None,
            start_function: None,
            start_defined_offset: None,
            reachable_defined_functions: None,
            function_dependency_graph: None,
            direct_runtime_init_functions: HashSet::new(),
            method_function_indices: BTreeMap::new(),
        }
    }

    pub(super) fn translate(mut self, bytes: &[u8]) -> Result<Translation> {
        validate_wasm_header(bytes)?;
        self.function_dependency_graph = Some(
            analyze_function_dependency_graph(bytes)
                .context("failed to analyze function dependencies for runtime init stubs")?,
        );
        self.reachable_defined_functions = Some(
            analyze_reachable_defined_functions(bytes)
                .context("failed to analyze reachable functions for size optimization")?,
        );
        self.parse_payloads(bytes)?;
        self.finalize()
    }
}

/// The 8-byte WebAssembly module preamble: magic `\0asm` + version 1.
const WASM_PREAMBLE: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

/// Reject obviously non-WASM input up front with a clear, actionable
/// message. Without this, a wrong/truncated file surfaces as the
/// misleading "failed to analyze function dependencies" context from the
/// first internal parse pass. Structurally-valid-but-broken modules still
/// fall through to the detailed `wasmparser` diagnostics downstream.
fn validate_wasm_header(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 8 {
        anyhow::bail!(
            "input is not a valid WebAssembly module: expected at least an 8-byte \
             header, got {} byte(s); pass a `.wasm` file built with \
             `cargo build --target wasm32-unknown-unknown`",
            bytes.len()
        );
    }
    if bytes[..4] != WASM_PREAMBLE[..4] {
        anyhow::bail!(
            "input is not a valid WebAssembly module: bad magic number \
             (expected 00 61 73 6d \"\\0asm\", got {:02x} {:02x} {:02x} {:02x})",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3]
        );
    }
    if bytes[4..8] != WASM_PREAMBLE[4..8] {
        anyhow::bail!(
            "unsupported WebAssembly version: expected 1 (01 00 00 00), got \
             {:02x} {:02x} {:02x} {:02x}",
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7]
        );
    }
    Ok(())
}

#[cfg(test)]
mod header_tests {
    use super::validate_wasm_header;

    #[test]
    fn rejects_short_input() {
        let err = validate_wasm_header(&[]).unwrap_err().to_string();
        assert!(err.contains("not a valid WebAssembly module"), "{err}");
        assert!(validate_wasm_header(&[0x00, 0x61, 0x73]).is_err());
    }

    #[test]
    fn rejects_bad_magic() {
        let err = validate_wasm_header(&[0xde, 0xad, 0xbe, 0xef, 0x01, 0x00, 0x00, 0x00])
            .unwrap_err()
            .to_string();
        assert!(err.contains("bad magic number"), "{err}");
    }

    #[test]
    fn rejects_bad_version() {
        let err = validate_wasm_header(&[0x00, 0x61, 0x73, 0x6d, 0x02, 0x00, 0x00, 0x00])
            .unwrap_err()
            .to_string();
        assert!(err.contains("unsupported WebAssembly version"), "{err}");
    }

    #[test]
    fn accepts_valid_preamble() {
        assert!(validate_wasm_header(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).is_ok());
    }
}
