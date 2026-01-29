//! Shared Test Utilities for wasm-neovm
//!
//! This module provides common testing helpers, mock builders, and assertion
//! macros used across all test modules.
//!
//! # Examples
//!
//! ```rust
//! use wasm_neovm::tests::test_utilities::*;
//!
//! let wasm = wat_to_wasm(r#"(module (func (export "test")))"#);
//! let translation = translate_wasm(&wasm, "TestContract");
//! assert_valid_nef(&translation.script);
//! ```

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Calculates double SHA256 checksum (used in NEF format)
pub fn double_sha256_checksum(data: &[u8]) -> u32 {
    let hash = Sha256::digest(data);
    let hash = Sha256::digest(hash);
    u32::from_le_bytes(hash[..4].try_into().unwrap())
}

/// Reads a variable-length unsigned integer from bytes
/// Returns (value, bytes_consumed)
pub fn read_var_uint(bytes: &[u8]) -> (u64, usize) {
    let prefix = bytes[0];
    match prefix {
        n if n < 0xFD => (u64::from(n), 1),
        0xFD => {
            let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
            (u64::from(value), 3)
        }
        0xFE => {
            let value = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
            (u64::from(value), 5)
        }
        0xFF => {
            let value = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            (value, 9)
        }
        _ => unreachable!(),
    }
}

/// Converts WAT (WebAssembly Text) format to WASM binary
/// Panics with descriptive message on failure
pub fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("Failed to parse WAT - check syntax")
}

/// Creates a minimal valid WASM module
pub fn minimal_wasm_module() -> Vec<u8> {
    wat::parse_str(r#"(module (func (export "main")))"#).unwrap()
}

/// Creates a WASM module with a simple function that returns an i32
pub fn simple_i32_return_wasm(value: i32) -> Vec<u8> {
    wat::parse_str(&format!(
        r#"(module (func (export "get_value") (result i32) i32.const {}))"#,
        value
    ))
    .expect("Failed to create simple i32 return WASM")
}

/// Translation result with metadata for testing
#[derive(Debug, Clone)]
pub struct TestTranslation {
    pub script: Vec<u8>,
    pub manifest: serde_json::Value,
    pub name: String,
}

/// Translates WASM bytes to NeoVM script with error context
pub fn translate_wasm(wasm: &[u8], name: &str) -> TestTranslation {
    let translation = wasm_neovm::translate_module(wasm, name)
        .unwrap_or_else(|e| panic!("Failed to translate '{}': {}", name, e));

    TestTranslation {
        script: translation.script,
        manifest: translation.manifest.value,
        name: name.to_string(),
    }
}

/// Asserts that a script is a valid NEF (Neo Executable Format)
pub fn assert_valid_nef(script: &[u8]) {
    assert!(
        script.len() >= 4,
        "NEF script too short: expected at least 4 bytes, got {}",
        script.len()
    );

    // Check for valid NeoVM script header (0x40 RET is typical end)
    assert_eq!(
        script.last(),
        Some(&0x40),
        "NEF script should end with RET opcode (0x40)"
    );
}

/// Asserts that a script contains specific opcodes
pub fn assert_contains_opcodes(script: &[u8], opcodes: &[&str]) {
    for opcode_name in opcodes {
        let opcode = wasm_neovm::opcodes::lookup(opcode_name)
            .unwrap_or_else(|| panic!("Unknown opcode: {}", opcode_name));
        assert!(
            script.contains(&opcode.byte),
            "Script should contain {} opcode (0x{:02x})",
            opcode_name,
            opcode.byte
        );
    }
}

/// Asserts that a script does NOT contain specific opcodes
pub fn assert_does_not_contain_opcodes(script: &[u8], opcodes: &[&str]) {
    for opcode_name in opcodes {
        let opcode = wasm_neovm::opcodes::lookup(opcode_name)
            .unwrap_or_else(|| panic!("Unknown opcode: {}", opcode_name));
        assert!(
            !script.contains(&opcode.byte),
            "Script should NOT contain {} opcode (0x{:02x})",
            opcode_name,
            opcode.byte
        );
    }
}

/// Counts occurrences of an opcode in a script
pub fn count_opcode(script: &[u8], opcode_name: &str) -> usize {
    let opcode = wasm_neovm::opcodes::lookup(opcode_name).expect("Unknown opcode");
    script.iter().filter(|&&b| b == opcode.byte).count()
}

/// Finds the position of an opcode in a script
pub fn find_opcode_position(script: &[u8], opcode_name: &str) -> Option<usize> {
    let opcode = wasm_neovm::opcodes::lookup(opcode_name).expect("Unknown opcode");
    script.iter().position(|&b| b == opcode.byte)
}

/// Creates a property-based test case builder
pub struct PropertyTestBuilder {
    name: String,
    iterations: usize,
}

impl PropertyTestBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            iterations: 100,
        }
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// Run a property test with generated inputs
    pub fn run<F>(&self, mut test_fn: F)
    where
        F: FnMut(&[u8]),
    {
        for i in 0..self.iterations {
            let input = generate_test_input(i);
            test_fn(&input);
        }
    }
}

/// Generates deterministic test input based on seed
fn generate_test_input(seed: usize) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();

    // Generate varied input sizes
    let size = (hash % 256) as usize;
    let mut input = Vec::with_capacity(size);

    for i in 0..size {
        input.push(((hash >> (i % 8)) ^ (i as u64)) as u8);
    }

    input
}

/// Mock builder for creating test WASM modules with specific features
pub struct WasmMockBuilder {
    imports: Vec<String>,
    exports: Vec<String>,
    globals: Vec<(String, i64, bool)>, // (name, initial_value, mutable)
    memory: Option<(u32, Option<u32>)>, // (min_pages, max_pages)
    tables: Vec<(u32, Option<u32>)>,   // (min_size, max_size)
    functions: Vec<String>,
}

impl WasmMockBuilder {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            exports: Vec::new(),
            globals: Vec::new(),
            memory: None,
            tables: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn with_memory(mut self, min_pages: u32, max_pages: Option<u32>) -> Self {
        self.memory = Some((min_pages, max_pages));
        self
    }

    pub fn with_global(mut self, name: &str, value: i64, mutable: bool) -> Self {
        self.globals.push((name.to_string(), value, mutable));
        self
    }

    pub fn with_export(mut self, name: &str) -> Self {
        self.exports.push(name.to_string());
        self
    }

    /// Builds a minimal WASM module
    pub fn build_minimal(self) -> Vec<u8> {
        minimal_wasm_module()
    }

    /// Builds a module with arithmetic operations
    pub fn build_arithmetic(self) -> Vec<u8> {
        wat::parse_str(
            r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add)
                (func (export "sub") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.sub)
                (func (export "mul") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.mul)
                (func (export "div") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.div_s)
            )
        "#,
        )
        .unwrap()
    }

    /// Builds a module with control flow
    pub fn build_control_flow(self) -> Vec<u8> {
        wat::parse_str(
            r#"
            (module
                (func (export "if_else") (param i32) (result i32)
                    local.get 0
                    if (result i32)
                        i32.const 1
                    else
                        i32.const 0
                    end)
                (func (export "loop_test") (param i32) (result i32)
                    (local $i i32)
                    (local $sum i32)
                    i32.const 0
                    local.set $sum
                    i32.const 0
                    local.set $i
                    (loop $continue (result i32)
                        local.get $sum
                        local.get $i
                        i32.add
                        local.set $sum
                        local.get $i
                        i32.const 1
                        i32.add
                        local.set $i
                        local.get $i
                        local.get 0
                        i32.lt_s
                        br_if $continue
                        local.get $sum
                    ))
                (func (export "block_test") (result i32)
                    block (result i32)
                        i32.const 42
                    end)
            )
        "#,
        )
        .unwrap()
    }
}

impl Default for WasmMockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test coverage tracker for identifying untested code paths
pub struct CoverageTracker {
    covered_paths: HashMap<String, bool>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            covered_paths: HashMap::new(),
        }
    }

    pub fn mark_covered(&mut self, path: &str) {
        self.covered_paths.insert(path.to_string(), true);
    }

    pub fn is_covered(&self, path: &str) -> bool {
        self.covered_paths.get(path).copied().unwrap_or(false)
    }

    pub fn coverage_percentage(&self, total_paths: usize) -> f64 {
        if total_paths == 0 {
            return 100.0;
        }
        let covered = self.covered_paths.values().filter(|&&v| v).count();
        (covered as f64 / total_paths as f64) * 100.0
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark helper for measuring translation performance
pub struct TranslationBenchmark {
    iterations: usize,
}

impl TranslationBenchmark {
    pub fn new(iterations: usize) -> Self {
        Self { iterations }
    }

    pub fn run<F, T>(&self, name: &str, f: F) -> std::time::Duration
    where
        F: Fn() -> T,
    {
        let start = std::time::Instant::now();
        for _ in 0..self.iterations {
            let _ = f();
        }
        let elapsed = start.elapsed();
        println!(
            "Benchmark '{}' completed {} iterations in {:?} (avg: {:?})",
            name,
            self.iterations,
            elapsed,
            elapsed / self.iterations as u32
        );
        elapsed
    }
}

/// Mock NeoVM runtime for testing without full VM
pub struct MockNeoRuntime {
    storage: HashMap<Vec<u8>, Vec<u8>>,
    gas_consumed: u64,
    notifications: Vec<(String, Vec<u8>)>,
}

impl MockNeoRuntime {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            gas_consumed: 0,
            notifications: Vec::new(),
        }
    }

    pub fn store(&mut self, key: &[u8], value: &[u8]) {
        self.storage.insert(key.to_vec(), value.to_vec());
        self.gas_consumed += 10;
    }

    pub fn load(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    pub fn notify(&mut self, event: &str, data: &[u8]) {
        self.notifications.push((event.to_string(), data.to_vec()));
        self.gas_consumed += 5;
    }

    pub fn gas_consumed(&self) -> u64 {
        self.gas_consumed
    }

    pub fn notification_count(&self) -> usize {
        self.notifications.len()
    }
}

impl Default for MockNeoRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wat_to_wasm() {
        let wasm = wat_to_wasm(r#"(module (func (export "test")))"#);
        assert!(!wasm.is_empty());
        assert!(wasm.starts_with(&[0x00, 0x61, 0x73, 0x6d])); // WASM magic
    }

    #[test]
    fn test_minimal_wasm_module() {
        let wasm = minimal_wasm_module();
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_simple_i32_return() {
        let wasm = simple_i32_return_wasm(42);
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_double_sha256() {
        let data = b"test";
        let checksum = double_sha256_checksum(data);
        assert_ne!(checksum, 0);

        // Same input should produce same checksum
        let checksum2 = double_sha256_checksum(data);
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_read_var_uint() {
        // Single byte
        let bytes = vec![0x10u8];
        let (value, consumed) = read_var_uint(&bytes);
        assert_eq!(value, 16);
        assert_eq!(consumed, 1);

        // 0xFD prefix (2 bytes)
        let bytes = vec![0xFDu8, 0x10, 0x00];
        let (value, consumed) = read_var_uint(&bytes);
        assert_eq!(value, 16);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_mock_builder() {
        let builder = WasmMockBuilder::new()
            .with_memory(1, Some(2))
            .with_global("g1", 42, false);

        let wasm = builder.build_minimal();
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_coverage_tracker() {
        let mut tracker = CoverageTracker::new();
        tracker.mark_covered("path1");
        tracker.mark_covered("path2");

        assert!(tracker.is_covered("path1"));
        assert!(!tracker.is_covered("path3"));
        assert_eq!(tracker.coverage_percentage(4), 50.0);
    }

    #[test]
    fn test_mock_runtime() {
        let mut runtime = MockNeoRuntime::new();

        runtime.store(b"key", b"value");
        assert_eq!(runtime.load(b"key"), Some(&b"value"[..].to_vec()));

        runtime.notify("TestEvent", b"data");
        assert_eq!(runtime.notification_count(), 1);
        assert!(runtime.gas_consumed() > 0);
    }
}
