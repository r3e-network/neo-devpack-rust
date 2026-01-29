//! Round 55: Advanced Integration Tests
//!
//! This module provides comprehensive integration tests that verify
//! end-to-end functionality across multiple components.

use std::io::Write;
use tempfile::NamedTempFile;
use wasm_neovm::{translate_module, translate_with_config, TranslationConfig};

/// Integration Test: Full contract compilation workflow
///
/// Tests the complete workflow from WASM to NEF output
#[test]
fn full_contract_compilation_workflow() {
    // Create a realistic WASM contract
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (global $counter (mut i32) (i32.const 0))
            
            (func (export "getCounter") (result i32)
                global.get $counter)
            
            (func (export "increment") (result i32)
                global.get $counter
                i32.const 1
                i32.add
                global.set $counter
                global.get $counter)
            
            (func (export "decrement") (result i32)
                global.get $counter
                i32.const 1
                i32.sub
                global.set $counter
                global.get $counter)
            
            (func (export "add") (param i32) (result i32)
                global.get $counter
                local.get 0
                i32.add
                global.set $counter
                global.get $counter)
        )"#,
    )
    .expect("Valid WAT");

    // Translate with default config
    let translation =
        translate_module(&wasm, "CounterContract").expect("Translation should succeed");

    // Verify script is valid
    assert!(!translation.script.is_empty(), "Script should not be empty");
    assert_eq!(
        translation.script.last(),
        Some(&0x40),
        "Script should end with RET"
    );

    // Verify manifest is valid JSON
    let manifest: serde_json::Value = translation.manifest.value.clone();

    // Verify ABI has all exported functions
    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    let method_names: Vec<_> = methods
        .iter()
        .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(
        method_names.contains(&"getCounter"),
        "Should export getCounter"
    );
    assert!(
        method_names.contains(&"increment"),
        "Should export increment"
    );
    assert!(
        method_names.contains(&"decrement"),
        "Should export decrement"
    );
    assert!(method_names.contains(&"add"), "Should export add");
}

/// Integration Test: Complex data structures
#[test]
fn complex_data_structure_handling() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (global $heap (mut i32) (i32.const 1024))
            
            ;; Allocate memory
            (func $alloc (param i32) (result i32)
                global.get $heap
                global.get $heap
                local.get 0
                i32.add
                global.set $heap)
            
            ;; Store i32 at address
            (func (export "storeI32") (param i32 i32)
                local.get 0
                local.get 1
                i32.store)
            
            ;; Load i32 from address
            (func (export "loadI32") (param i32) (result i32)
                local.get 0
                i32.load)
            
            ;; Store i64 at address
            (func (export "storeI64") (param i32 i64)
                local.get 0
                local.get 1
                i64.store)
            
            ;; Load i64 from address
            (func (export "loadI64") (param i32) (result i64)
                local.get 0
                i64.load)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "MemoryOps").expect("Translation should succeed");

    assert!(!translation.script.is_empty(), "Script should contain code");

    // Verify manifest
    let manifest: serde_json::Value = translation.manifest.value.clone();

    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    assert_eq!(methods.len(), 4, "Should have 4 exported functions");
}

/// Integration Test: Recursive function handling
#[test]
fn recursive_function_translation() {
    let wasm = wat::parse_str(
        r#"(module
            ;; Recursive factorial
            (func $fact (param i64) (result i64)
                local.get 0
                i64.const 1
                i64.le_s
                if (result i64)
                    i64.const 1
                else
                    local.get 0
                    local.get 0
                    i64.const 1
                    i64.sub
                    call $fact
                    i64.mul
                end)
            
            (func (export "factorial") (param i64) (result i64)
                local.get 0
                call $fact)
            
            ;; Recursive fibonacci
            (func $fib (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.lt_s
                if (result i32)
                    local.get 0
                else
                    local.get 0
                    i32.const 1
                    i32.sub
                    call $fib
                    local.get 0
                    i32.const 2
                    i32.sub
                    call $fib
                    i32.add
                end)
            
            (func (export "fibonacci") (param i32) (result i32)
                local.get 0
                call $fib)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "RecursiveMath").expect("Translation should succeed");

    assert!(
        !translation.script.is_empty(),
        "Recursive functions should translate"
    );
}

/// Integration Test: Multiple return values via memory
#[test]
fn multiple_return_values_simulation() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            
            ;; Divide and return quotient and remainder
            (func (export "divmod") (param i32 i32 i32)
                ;; param 0 = dividend, param 1 = divisor, param 2 = result address
                local.get 2
                local.get 0
                local.get 1
                i32.div_s
                i32.store
                
                local.get 2
                i32.const 4
                i32.add
                local.get 0
                local.get 1
                i32.rem_s
                i32.store)
            
            ;; Swap two values via memory
            (func (export "swapValues") (param i32 i32)
                (local $temp i32)
                local.get 0
                i32.load
                local.set $temp
                
                local.get 0
                local.get 1
                i32.load
                i32.store
                
                local.get 1
                local.get $temp
                i32.store)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "MultiReturn").expect("Translation should succeed");

    assert!(!translation.script.is_empty());
}

/// Integration Test: String handling simulation
#[test]
fn string_handling_simulation() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (data (i32.const 1024) "Hello, Neo!")
            
            ;; Get string length
            (func (export "strlen") (param i32) (result i32)
                (local $len i32)
                i32.const 0
                local.set $len
                loop $count
                    local.get 0
                    local.get $len
                    i32.add
                    i32.load8_u
                    if
                        local.get $len
                        i32.const 1
                        i32.add
                        local.set $len
                        br $count
                    end
                end
                local.get $len)
            
            ;; Copy string
            (func (export "strcpy") (param i32 i32)
                loop $copy
                    local.get 1
                    local.get 0
                    i32.load8_u
                    i32.store8
                    
                    local.get 0
                    i32.load8_u
                    i32.eqz
                    if
                        return
                    end
                    
                    local.get 0
                    i32.const 1
                    i32.add
                    local.set 0
                    local.get 1
                    i32.const 1
                    i32.add
                    local.set 1
                    br $copy
                end)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "StringOps").expect("Translation should succeed");

    assert!(!translation.script.is_empty());
}

/// Integration Test: CLI workflow simulation
#[test]
fn cli_workflow_simulation() {
    // Create a WASM file
    let wasm = wat::parse_str(
        r#"(module
            (func (export "main") (result i32)
                i32.const 42)
        )"#,
    )
    .expect("Valid WAT");

    // Simulate reading from file
    let mut temp_file = NamedTempFile::new().expect("Should create temp file");
    temp_file.write_all(&wasm).expect("Should write WASM");

    // Read back and translate
    let wasm_from_file = std::fs::read(temp_file.path()).expect("Should read file");

    let translation =
        translate_module(&wasm_from_file, "MainContract").expect("Translation should succeed");

    assert!(!translation.script.is_empty());

    // Cleanup happens automatically when temp_file is dropped
}

/// Integration Test: Manifest overlay merging
#[test]
fn manifest_overlay_merging() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "transfer") (param i32 i32) (result i32)
                i32.const 1)
            (func (export "balanceOf") (param i32) (result i32)
                i32.const 0)
        )"#,
    )
    .expect("Valid WAT");

    // Translate with config
    let config = TranslationConfig::new("TokenContract");
    let translation =
        translate_with_config(&wasm, config).expect("Translation with config should succeed");

    // Verify manifest structure
    let manifest: serde_json::Value = translation.manifest.value.clone();

    assert!(
        manifest.get("name").is_some(),
        "Manifest should have name field"
    );

    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");
    assert_eq!(methods.len(), 2, "Should have 2 methods");
}

/// Integration Test: Error propagation
#[test]
fn error_propagation_across_components() {
    // Test various error conditions

    // 1. Invalid WASM magic
    let invalid_magic = vec![0x00, 0x00, 0x00, 0x00];
    let result = translate_module(&invalid_magic, "Invalid");
    assert!(result.is_err(), "Should fail with invalid magic");

    // 2. Truncated WASM
    let truncated = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    let result = translate_module(&truncated, "Truncated");
    assert!(result.is_err(), "Should fail with truncated WASM");

    // 3. Valid WASM should succeed
    let valid = wat::parse_str(r#"(module (func (export "test")))"#).expect("Valid WAT");
    let result = translate_module(&valid, "Valid");
    assert!(result.is_ok(), "Should succeed with valid WASM");
}

/// Integration Test: Large module handling
#[test]
fn large_module_translation() {
    // Generate a WASM with many functions
    let mut funcs = String::new();
    for i in 0..100 {
        funcs.push_str(&format!(
            r#"(func (export "func{i}") (result i32) i32.const {i})"#
        ));
    }

    let wat = format!(r#"(module {})"#, funcs);
    let wasm = wat::parse_str(&wat).expect("Valid WAT");

    let translation =
        translate_module(&wasm, "LargeModule").expect("Translation of large module should succeed");

    // Verify manifest has all functions
    let manifest: serde_json::Value = translation.manifest.value.clone();

    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    assert_eq!(methods.len(), 100, "Should have 100 exported functions");
}

/// Integration Test: Cross-module calling patterns
#[test]
fn cross_module_calling_patterns() {
    // Internal function calls
    let wasm = wat::parse_str(
        r#"(module
            (func $helper (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.add)
            
            (func (export "compute") (param i32) (result i32)
                local.get 0
                call $helper
                local.get 0
                call $helper
                i32.add)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "InternalCalls").expect("Translation should succeed");

    assert!(!translation.script.is_empty());
}

/// Integration Test: Table and indirect calls
#[test]
fn table_indirect_call_handling() {
    let wasm = wat::parse_str(
        r#"(module
            (type $t0 (func (param i32) (result i32)))
            (table 3 funcref)
            (elem (i32.const 0) $f1 $f2 $f3)
            
            (func $f1 (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)
            
            (func $f2 (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.add)
            
            (func $f3 (param i32) (result i32)
                local.get 0
                i32.const 3
                i32.add)
            
            (func (export "callIndirect") (param i32 i32) (result i32)
                local.get 1
                local.get 0
                call_indirect (type $t0))
        )"#,
    )
    .expect("Valid WAT");

    // This may fail or succeed depending on funcref support
    let _result = translate_module(&wasm, "IndirectCalls");
    // We test that the translator handles this gracefully
}

/// Integration Test: Bulk memory operations
#[test]
fn bulk_memory_operations() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (data "test data")
            
            (func (export "fill") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.fill)
            
            (func (export "copy") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.copy)
        )"#,
    )
    .expect("Valid WAT");

    let result = translate_module(&wasm, "BulkMemory");
    // May succeed or fail based on bulk memory support
    let _ = result;
}

/// Integration Test: Module with custom sections
#[test]
fn module_with_custom_sections() {
    let wasm = wat::parse_str(
        r#"(module
            (@custom "name" "test contract")
            (@custom "version" "1.0.0")
            
            (func (export "version") (result i32)
                i32.const 1)
        )"#,
    )
    .expect("Valid WAT");

    let translation =
        translate_module(&wasm, "CustomSections").expect("Translation should succeed");

    assert!(!translation.script.is_empty());
}
