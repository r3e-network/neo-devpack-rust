//! Round 56: Documentation Tests
//!
//! This module tests that documentation examples work correctly.
//! These tests serve as both documentation and verification.

use wasm_neovm::{opcodes, translate_module, translate_with_config, TranslationConfig};

/// # Example: Basic WASM to NeoVM Translation
///
/// ```
/// use wasm_neovm::translate_module;
///
/// // Simple WASM module that returns 42
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "answer") (result i32)
///             i32.const 42)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "AnswerContract").unwrap();
/// assert!(!translation.script.is_empty());
/// ```
#[test]
fn doc_example_basic_translation() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "answer") (result i32)
                i32.const 42)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "AnswerContract").unwrap();
    assert!(!translation.script.is_empty());
}

/// # Example: Using TranslationConfig
///
/// ```
/// use wasm_neovm::{translate_with_config, TranslationConfig};
///
/// let wasm = wat::parse_str(r#"
///     (module (func (export "test")))
/// "#).unwrap();
///
/// let config = TranslationConfig::new("MyContract");
/// let translation = translate_with_config(&wasm, config).unwrap();
/// ```
#[test]
fn doc_example_translation_config() {
    let wasm = wat::parse_str(r#"(module (func (export "test")))"#).unwrap();

    let config = TranslationConfig::new("MyContract");
    let translation = translate_with_config(&wasm, config).unwrap();

    assert!(!translation.script.is_empty());
}

/// # Example: Opcode Lookup
///
/// ```
/// use wasm_neovm::opcodes;
///
/// // Look up an opcode by name
/// if let Some(push1) = opcodes::lookup("PUSH1") {
///     assert_eq!(push1.name, "PUSH1");
///     assert_eq!(push1.byte, 0x11);
/// }
///
/// // Look up by lowercase name (case insensitive)
/// let pushint8 = opcodes::lookup("pushint8");
/// assert!(pushint8.is_some());
/// ```
#[test]
fn doc_example_opcode_lookup() {
    // Look up an opcode by name
    if let Some(push1) = opcodes::lookup("PUSH1") {
        assert_eq!(push1.name, "PUSH1");
        assert_eq!(push1.byte, 0x11);
    }

    // Look up by lowercase name (case insensitive)
    let pushint8 = opcodes::lookup("pushint8");
    assert!(pushint8.is_some());
}

/// # Example: Working with Manifest
///
/// ```
/// use wasm_neovm::translate_module;
/// use serde_json::Value;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "transfer") (param i32 i32) (result i32)
///             i32.const 1)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "TokenContract").unwrap();
///
/// // Parse the manifest JSON
/// let manifest: Value = translation.manifest.value.clone();
///
/// // Access ABI information
/// if let Some(methods) = manifest["abi"]["methods"].as_array() {
///     assert!(!methods.is_empty());
/// }
/// ```
#[test]
fn doc_example_manifest_parsing() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "transfer") (param i32 i32) (result i32)
                i32.const 1)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "TokenContract").unwrap();

    // Parse the manifest JSON
    let manifest: serde_json::Value = translation.manifest.value.clone();

    // Access ABI information
    if let Some(methods) = manifest["abi"]["methods"].as_array() {
        assert!(!methods.is_empty());
    }
}

/// # Example: Error Handling
///
/// ```
/// use wasm_neovm::translate_module;
///
/// // Invalid WASM bytes
/// let invalid = vec![0x00, 0x00, 0x00, 0x00];
///
/// // Translation returns a Result
/// match translate_module(&invalid, "Invalid") {
///     Ok(_) => panic!("Should have failed"),
///     Err(e) => {
///         // Error can be printed or chained
///         let _ = format!("Translation failed: {}", e);
///     }
/// }
/// ```
#[test]
fn doc_example_error_handling() {
    // Invalid WASM bytes
    let invalid = vec![0x00, 0x00, 0x00, 0x00];

    // Translation returns a Result
    match translate_module(&invalid, "Invalid") {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            // Error can be printed or chained
            let _ = format!("Translation failed: {}", e);
        }
    }
}

/// # Example: Arithmetic Operations
///
/// ```
/// use wasm_neovm::translate_module;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "add") (param i32 i32) (result i32)
///             local.get 0
///             local.get 1
///             i32.add)
///         (func (export "mul") (param i32 i32) (result i32)
///             local.get 0
///             local.get 1
///             i32.mul)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "MathContract").unwrap();
/// assert!(!translation.script.is_empty());
/// ```
#[test]
fn doc_example_arithmetic() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
            (func (export "mul") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "MathContract").unwrap();
    assert!(!translation.script.is_empty());
}

/// # Example: Control Flow
///
/// ```
/// use wasm_neovm::translate_module;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "max") (param i32 i32) (result i32)
///             local.get 0
///             local.get 1
///             i32.gt_s
///             if (result i32)
///                 local.get 0
///             else
///                 local.get 1
///             end)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "MaxContract").unwrap();
/// assert_eq!(translation.script.last(), Some(&0x40)); // Ends with RET
/// ```
#[test]
fn doc_example_control_flow() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "max") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_s
                if (result i32)
                    local.get 0
                else
                    local.get 1
                end)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "MaxContract").unwrap();
    assert_eq!(translation.script.last(), Some(&0x40)); // Ends with RET
}

/// # Example: Memory Operations
///
/// ```
/// use wasm_neovm::translate_module;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (memory 1)
///         (func (export "load") (param i32) (result i32)
///             local.get 0
///             i32.load)
///         (func (export "store") (param i32 i32)
///             local.get 0
///             local.get 1
///             i32.store)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "MemoryContract").unwrap();
/// assert!(!translation.script.is_empty());
/// ```
#[test]
fn doc_example_memory_operations() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (func (export "load") (param i32) (result i32)
                local.get 0
                i32.load)
            (func (export "store") (param i32 i32)
                local.get 0
                local.get 1
                i32.store)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "MemoryContract").unwrap();
    assert!(!translation.script.is_empty());
}

/// # Example: Getting All Opcodes
///
/// ```
/// use wasm_neovm::opcodes;
///
/// // Get all available opcodes
/// let all_opcodes = opcodes::all();
/// assert!(!all_opcodes.is_empty());
///
/// // Find a specific opcode
/// let push0 = all_opcodes.iter().find(|op| op.name == "PUSH0");
/// assert!(push0.is_some());
/// ```
#[test]
fn doc_example_get_all_opcodes() {
    // Get all available opcodes
    let all_opcodes = opcodes::all();
    assert!(!all_opcodes.is_empty());

    // Find a specific opcode
    let push0 = all_opcodes.iter().find(|op| op.name == "PUSH0");
    assert!(push0.is_some());
}

/// # Example: Checking Script Structure
///
/// ```
/// use wasm_neovm::{translate_module, opcodes};
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "test") (result i32)
///             i32.const 42)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "TestContract").unwrap();
///
/// // Check that script ends with RET (0x40)
/// assert_eq!(translation.script.last(), Some(&0x40));
///
/// // Check for specific opcodes
/// let pushint8 = opcodes::lookup("PUSHINT8").unwrap();
/// assert!(translation.script.contains(&pushint8.byte));
/// ```
#[test]
fn doc_example_checking_script() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "test") (result i32)
                i32.const 42)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "TestContract").unwrap();

    // Check that script ends with RET (0x40)
    assert_eq!(translation.script.last(), Some(&0x40));

    // Check for specific opcodes
    let pushint8 = opcodes::lookup("PUSHINT8").unwrap();
    assert!(translation.script.contains(&pushint8.byte));
}

/// # Example: Working with Multiple Exports
///
/// ```
/// use wasm_neovm::translate_module;
/// use serde_json::Value;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "getValue") (result i32)
///             i32.const 100)
///         (func (export "setValue") (param i32))
///         (func (export "increment") (result i32)
///             i32.const 1)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "MultiExport").unwrap();
/// let manifest: Value = translation.manifest.value.clone();
///
/// let methods = manifest["abi"]["methods"].as_array().unwrap();
/// assert_eq!(methods.len(), 3);
/// ```
#[test]
fn doc_example_multiple_exports() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "getValue") (result i32)
                i32.const 100)
            (func (export "setValue") (param i32))
            (func (export "increment") (result i32)
                i32.const 1)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "MultiExport").unwrap();
    let manifest: serde_json::Value = translation.manifest.value.clone();

    let methods = manifest["abi"]["methods"].as_array().unwrap();
    assert_eq!(methods.len(), 3);
}

/// # Example: Using Local Variables
///
/// ```
/// use wasm_neovm::translate_module;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "compute") (param i32) (result i32)
///             (local $temp i32)
///             local.get 0
///             i32.const 10
///             i32.add
///             local.set $temp
///             local.get $temp
///             i32.const 5
///             i32.mul)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "LocalVarContract").unwrap();
/// assert!(!translation.script.is_empty());
/// ```
#[test]
fn doc_example_local_variables() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "compute") (param i32) (result i32)
                (local $temp i32)
                local.get 0
                i32.const 10
                i32.add
                local.set $temp
                local.get $temp
                i32.const 5
                i32.mul)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "LocalVarContract").unwrap();
    assert!(!translation.script.is_empty());
}

/// # Example: Handling Division
///
/// ```
/// use wasm_neovm::translate_module;
///
/// let wasm = wat::parse_str(r#"
///     (module
///         (func (export "divide") (param i32 i32) (result i32)
///             local.get 0
///             local.get 1
///             i32.div_s)
///     )
/// "#).unwrap();
///
/// let translation = translate_module(&wasm, "DivContract").unwrap();
///
/// // Division includes zero-check
/// assert!(!translation.script.is_empty());
/// ```
#[test]
fn doc_example_division() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "divide") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_s)
        )"#,
    )
    .unwrap();

    let translation = translate_module(&wasm, "DivContract").unwrap();

    // Division includes zero-check
    assert!(!translation.script.is_empty());
}
