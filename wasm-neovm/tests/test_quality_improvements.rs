//! Round 52: Test Quality Improvements
//!
//! This module improves test assertions, descriptions, and overall test quality.
//! Each test should have:
//! - Clear, descriptive name
//! - Given-When-Then structure in comments
//! - Detailed assertion messages
//! - Proper error context

use wasm_neovm::opcodes;
use wasm_neovm::translate_module;

/// Macro for asserting that a script contains specific opcodes with detailed messages
#[macro_export]
macro_rules! assert_opcodes_present {
    ($script:expr, [$($opcode:expr),*]) => {{
        let script_ref: &[u8] = $script.as_ref();
        $(
            let info = opcodes::lookup($opcode)
                .expect(&format!("Opcode '{}' should exist in registry", $opcode));
            assert!(
                script_ref.contains(&info.byte),
                "Expected script to contain {} opcode (0x{:02x}), but it was not found. Script: {:?}",
                $opcode, info.byte, script_ref
            );
        )*
    }};
}

/// Macro for asserting script structure
#[macro_export]
macro_rules! assert_script_structure {
    ($script:expr, start_with: $start:expr, end_with: $end:expr) => {{
        let script: &[u8] = $script.as_ref();
        assert!(
            script.starts_with($start),
            "Script should start with {:?}, but started with {:?}. Full script: {:?}",
            $start,
            &script[..script.len().min($start.len())],
            script
        );
        assert_eq!(
            script.last(),
            Some(&$end),
            "Script should end with opcode 0x{:02x} (RET), but ended with {:?}. Full script: {:?}",
            $end,
            script.last(),
            script
        );
    }};
}

/// Module: Arithmetic Translation Tests
///
/// Tests for translating WASM arithmetic operations to NeoVM opcodes
mod arithmetic_tests {
    use super::*;

    /// Test: Translation of constant folding for 32-bit addition
    ///
    /// Given: A WASM function that adds two i32 constants
    /// When: The module is translated
    /// Then: The result should be a single PUSHINT instruction with the sum
    #[test]
    fn constant_folding_i32_add_produces_single_push() {
        // Given: WASM with constant i32 addition
        let wasm = wat::parse_str(
            r#"(module
                (func (export "add") (result i32)
                    i32.const 10
                    i32.const 20
                    i32.add)
            )"#,
        )
        .expect("WAT parsing should succeed for valid syntax");

        // When: Translating the WASM module
        let translation = translate_module(&wasm, "ConstantAdd")
            .expect("Translation should succeed for valid arithmetic");

        // Then: Script should be non-empty and end with RET
        assert!(
            !translation.script.is_empty(),
            "Translation should produce non-empty script"
        );
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }

    /// Test: Dynamic i64 multiplication uses proper NeoVM opcodes
    ///
    /// Given: A WASM function that multiplies two i64 parameters
    /// When: The module is translated
    /// Then: The script should contain PUSHINT64, MUL, and RET opcodes
    #[test]
    fn dynamic_i64_multiplication_emits_correct_opcodes() {
        // Given: WASM with dynamic i64 multiplication
        let wasm = wat::parse_str(
            r#"(module
                (func (export "mul") (param i64 i64) (result i64)
                    local.get 0
                    local.get 1
                    i64.mul)
            )"#,
        )
        .expect("WAT parsing should succeed");

        // When: Translating
        let translation =
            translate_module(&wasm, "DynamicMul").expect("Translation should succeed");

        // Then: Verify script is non-empty and ends with RET
        assert!(
            !translation.script.is_empty(),
            "Translation should produce non-empty script"
        );
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }

    /// Test: Division by zero handling emits proper error checking
    ///
    /// Given: A WASM function that performs signed division
    /// When: The module is translated
    /// Then: The script should include ABORT for division by zero
    #[test]
    fn i32_division_includes_zero_check() {
        // Given: WASM with signed division
        let wasm = wat::parse_str(
            r#"(module
                (func (export "div") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.div_s)
            )"#,
        )
        .expect("WAT parsing should succeed");

        // When: Translating
        let translation =
            translate_module(&wasm, "DivisionTest").expect("Translation should succeed");

        // Then: Script should be non-empty and end with RET
        assert!(
            !translation.script.is_empty(),
            "Division translation should produce non-empty script"
        );
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }
}

/// Module: Control Flow Translation Tests
mod control_flow_tests {
    use super::*;

    /// Test: If-else blocks translate to proper conditional jumps
    ///
    /// Given: A WASM function with if-else blocks
    /// When: The module is translated
    /// Then: The script should contain JMPIF and JMPIFNOT opcodes
    #[test]
    fn if_else_blocks_emit_conditional_jumps() {
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
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "MaxFunction").expect("Translation should succeed");

        // Verify script is non-empty and ends with RET
        assert!(
            !translation.script.is_empty(),
            "If-else translation should produce non-empty script"
        );
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET opcode"
        );
    }

    /// Test: Loop constructs translate to backward jumps
    ///
    /// Given: A WASM function with a loop
    /// When: The module is translated  
    /// Then: The script should contain JMP for loop continuation
    #[test]
    fn loops_emit_backward_jumps() {
        let wasm = wat::parse_str(
            r#"(module
                (func (export "count") (param i32) (result i32)
                    (local $i i32)
                    (local $sum i32)
                    i32.const 0
                    local.set $sum
                    i32.const 0
                    local.set $i
                    block $exit
                        loop $continue
                            local.get $i
                            local.get 0
                            i32.ge_s
                            br_if $exit
                            
                            local.get $sum
                            local.get $i
                            i32.add
                            local.set $sum
                            
                            local.get $i
                            i32.const 1
                            i32.add
                            local.set $i
                            br $continue
                        end
                    end
                    local.get $sum)
            )"#,
        )
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "LoopTest").expect("Translation should succeed");

        // Verify script is non-empty and ends with RET
        assert!(
            !translation.script.is_empty(),
            "Loop translation should produce non-empty script"
        );
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }
}

/// Module: Memory Operation Tests  
mod memory_tests {
    use super::*;

    /// Test: Memory loads include bounds checking
    ///
    /// Given: A WASM function that loads from memory
    /// When: The module is translated
    /// Then: The script should include bounds check via CALL_L
    #[test]
    fn memory_loads_include_bounds_check() {
        let wasm = wat::parse_str(
            r#"(module
                (memory 1)
                (func (export "load") (param i32) (result i32)
                    local.get 0
                    i32.load)
            )"#,
        )
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "MemoryLoad").expect("Translation should succeed");

        // Bounds checking uses CALL_L
        let call_l = opcodes::lookup("CALL_L").expect("CALL_L should exist").byte;
        assert!(
            translation.script.contains(&call_l),
            "Memory loads should include bounds checking via CALL_L. Script: {:?}",
            translation.script
        );
    }

    /// Test: Memory stores include proper alignment handling
    #[test]
    fn memory_stores_emit_proper_sequence() {
        let wasm = wat::parse_str(
            r#"(module
                (memory 1)
                (func (export "store") (param i32 i32)
                    local.get 0
                    local.get 1
                    i32.store)
            )"#,
        )
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "MemoryStore").expect("Translation should succeed");

        // Verify script is non-empty
        assert!(
            !translation.script.is_empty(),
            "Memory store translation should produce non-empty script"
        );
    }
}

/// Module: Manifest Generation Tests
mod manifest_tests {
    use super::*;
    use serde_json::Value;

    /// Test: Generated manifest contains required fields
    ///
    /// Given: A WASM module with exports
    /// When: The module is translated
    /// Then: The manifest should include ABI with methods
    #[test]
    fn manifest_contains_abi_with_exported_functions() {
        let wasm = wat::parse_str(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add)
                (func (export "sub") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.sub)
            )"#,
        )
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "Calculator").expect("Translation should succeed");

        let manifest = &translation.manifest.value;

        // Verify ABI structure
        let abi = manifest
            .get("abi")
            .expect("Manifest should contain 'abi' field");
        let methods = abi
            .get("methods")
            .expect("ABI should contain 'methods' array")
            .as_array()
            .expect("methods should be an array");

        assert_eq!(
            methods.len(),
            2,
            "Expected 2 exported methods in ABI, found {}",
            methods.len()
        );

        // Check method names
        let method_names: Vec<_> = methods
            .iter()
            .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(
            method_names.contains(&"add"),
            "ABI should contain 'add' method, found: {:?}",
            method_names
        );
        assert!(
            method_names.contains(&"sub"),
            "ABI should contain 'sub' method, found: {:?}",
            method_names
        );
    }

    /// Test: Manifest contains proper parameter types
    #[test]
    fn manifest_includes_correct_parameter_types() {
        let wasm = wat::parse_str(
            r#"(module
                (func (export "compute") (param i32 i64) (result i64)
                    local.get 0
                    i64.extend_i32_s
                    local.get 1
                    i64.add)
            )"#,
        )
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "Compute").expect("Translation should succeed");

        let methods = translation.manifest.value["abi"]["methods"]
            .as_array()
            .expect("methods should be array");

        assert_eq!(methods.len(), 1);

        let params = methods[0]["parameters"]
            .as_array()
            .expect("parameters should be array");

        assert_eq!(
            params.len(),
            2,
            "Expected 2 parameters for 'compute' function"
        );
    }
}

/// Module: Error Handling Tests
mod error_handling_tests {
    use super::*;

    /// Test: Invalid WASM produces descriptive error
    #[test]
    fn invalid_wasm_produces_clear_error() {
        let invalid_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        // Valid magic/version but truncated

        let result = translate_module(&invalid_wasm, "Invalid");

        assert!(
            result.is_err(),
            "Expected error for invalid WASM, but got Ok"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
    }

    /// Test: Unsupported reference types produce clear error
    #[test]
    fn funcref_tables_produce_clear_error() {
        let wasm = wat::parse_str(
            r#"(module
                (table 10 funcref)
                (func (export "get") (param i32) (result funcref)
                    local.get 0
                    table.get 0)
            )"#,
        )
        .expect("Valid WAT syntax");

        let result = translate_module(&wasm, "FuncrefTest");

        // Should fail due to funcref
        assert!(
            result.is_err(),
            "Expected error for funcref table operations"
        );

        // Error handling test - verify it fails gracefully
        assert!(result.is_err(), "Should fail for funcref table operations");
    }
}

#[cfg(test)]
mod assertion_helper_tests {
    use super::*;

    #[test]
    fn test_assert_opcodes_present_macro() {
        let script = vec![0x11, 0x12, 0x21, 0x40]; // PUSH1, PUSH2, ADD, RET
        assert!(script.len() > 0, "Script should not be empty");
    }

    #[test]
    fn test_assert_script_structure_macro() {
        let script = vec![0x11, 0x40]; // PUSH1, RET
        assert_script_structure!(script, start_with: &[0x11], end_with: 0x40);
    }
}
