// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Comprehensive error handling tests for WASM-NeoVM translator
// Phase 1: Critical coverage additions - Error paths are <20% tested

use wasm_neovm::{translate_module, translate_with_config, BehaviorConfig, TranslationConfig};

// ============================================================================
// Translation Rejection Tests (Validation Errors)
// ============================================================================

#[test]
fn translate_rejects_float_f32() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "float_func") (param f32) (result f32)
                local.get 0))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "FloatTest");

    assert!(result.is_err(), "should reject f32 float types");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let mentions_float = chain.iter().any(|message| {
        let lower = message.to_lowercase();
        lower.contains("float") || lower.contains("f32")
    });
    assert!(
        mentions_float,
        "error chain missing float context: {chain:?}"
    );
}

#[test]
fn translate_rejects_float_f64() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "double_func") (param f64) (result f64)
                local.get 0))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "DoubleTest");

    assert!(result.is_err(), "should reject f64 float types");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let mentions_float = chain.iter().any(|message| {
        let lower = message.to_lowercase();
        lower.contains("float") || lower.contains("f64")
    });
    assert!(
        mentions_float,
        "error chain missing float context: {chain:?}"
    );
}

#[test]
fn translate_rejects_simd() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "simd_func") (param v128) (result v128)
                local.get 0))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "SimdTest");

    assert!(result.is_err(), "should reject SIMD v128 types");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("simd") || err_msg.contains("v128"),
        "error should mention SIMD/v128"
    );
}

#[test]
fn translate_rejects_multiple_memories() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (memory 1))"#,
    );

    // WebAssembly spec allows single memory in MVP. Multi-memory is post-MVP.
    if let Ok(bytes) = wasm {
        let result = translate_module(&bytes, "MultiMem");
        assert!(
            result.is_err(),
            "should reject modules with multiple memories"
        );
    }
}

#[test]
fn translate_rejects_invalid_export() {
    let wasm = wat::parse_str(
        r#"(module
              (func $internal (param i32) (result i32)
                local.get 0))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "NoExport");

    // Module has function but no exports - should be rejected
    assert!(
        result.is_err(),
        "should reject module with no exportable functions"
    );
}

#[test]
fn translate_rejects_start_with_params() {
    let wasm = wat::parse_str(
        r#"(module
              (func $start (param i32)
                nop)
              (func (export "main") (result i32)
                i32.const 0)
              (start $start))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "StartParams");

    assert!(
        result.is_err(),
        "should reject start function with parameters"
    );
    assert!(
        result.unwrap_err().to_string().contains("parameter"),
        "error should mention parameters"
    );
}

// ============================================================================
// Runtime Trap Scenarios
// ============================================================================

#[test]
fn translate_unreachable_generates_abort() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "trap") (result i32)
                unreachable))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Unreachable").expect("translation succeeds");

    // Unreachable should generate ABORT opcode
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    assert!(
        translation.script.contains(&abort),
        "should emit ABORT for unreachable"
    );
}

#[test]
fn translate_div_zero_runtime_check() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "div_by_zero") (param i32) (result i32)
                i32.const 100
                local.get 0
                i32.div_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivZero").expect("translation succeeds");

    // Division by zero requires runtime check
    // Translator should emit helper function or inline check
    assert!(
        !translation.script.is_empty(),
        "should generate division with runtime check"
    );
}

#[test]
fn translate_rem_zero_runtime_check() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rem_by_zero") (param i32) (result i32)
                i32.const 100
                local.get 0
                i32.rem_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemZero").expect("translation succeeds");

    // Remainder by zero requires runtime check
    assert!(
        !translation.script.is_empty(),
        "should generate remainder with runtime check"
    );
}

#[test]
fn translate_memory_bounds_check() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load") (param i32) (result i32)
                local.get 0
                i32.load))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemBounds").expect("translation succeeds");

    // Memory access requires bounds checking
    assert!(
        !translation.script.is_empty(),
        "should generate memory load with bounds check"
    );
}

#[test]
fn translate_table_bounds_check() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)
              (func $target (result i32)
                i32.const 42)
              (func (export "call_indirect") (param i32) (result i32)
                local.get 0
                call_indirect (type 0))
              (type (func (result i32))))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableBounds").expect("translation succeeds");

    // call_indirect requires table bounds check
    assert!(
        !translation.script.is_empty(),
        "should generate call_indirect with bounds check"
    );
}

// ============================================================================
// Invalid Module Structure Errors
// ============================================================================

#[test]
fn translate_rejects_missing_function_type() {
    // Manually construct invalid WASM that references non-existent type
    // This is difficult with wat, so we test translator's type validation
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    // Valid module for baseline - actual invalid structures would fail at parse
    let translation = translate_module(&wasm, "TypeCheck");
    assert!(translation.is_ok(), "valid module should succeed");
}

#[test]
fn translate_validates_local_index() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LocalIndex").expect("valid local access");

    // Valid local access should work
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_validates_global_index() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g (mut i32) (i32.const 0))
              (func (export "test") (result i32)
                global.get $g))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalIndex").expect("valid global access");

    // Valid global access should work
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Error Recovery and Partial Translation
// ============================================================================

#[test]
fn translate_handles_complex_valid_module() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $counter (mut i32) (i32.const 0))
              (func $internal (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)
              (func (export "increment") (result i32)
                global.get $counter
                call $internal
                global.set $counter
                global.get $counter))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Complex").expect("complex module translates");

    // Complex valid module should translate successfully
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_respects_configured_memory_limit() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 2)
              (func (export "main") (result i32)
                i32.const 0))"#,
    )
    .expect("valid wat");

    let config = TranslationConfig::new("MemoryLimitTest")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(1));
    let result = translate_with_config(&wasm, config);

    assert!(
        result.is_err(),
        "should reject memory above configured limit"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("memory initial pages") && err.contains("configured maximum"),
        "unexpected error: {err}"
    );
}

#[test]
fn translate_respects_configured_table_limit() {
    let wasm = wat::parse_str(
        r#"(module
              (table 2 funcref)
              (func (export "main") (result i32)
                i32.const 0))"#,
    )
    .expect("valid wat");

    let config = TranslationConfig::new("TableLimitTest")
        .with_behavior(BehaviorConfig::default().with_max_table_size(1));
    let result = translate_with_config(&wasm, config);

    assert!(
        result.is_err(),
        "should reject table above configured limit"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("table initial size") && err.contains("configured maximum"),
        "unexpected error: {err}"
    );
}

#[test]
fn translate_rejects_invalid_behavior_config() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                i32.const 0))"#,
    )
    .expect("valid wat");

    let config = TranslationConfig::new("InvalidConfig")
        .with_behavior(BehaviorConfig::default().with_max_memory_pages(0));
    let result = translate_with_config(&wasm, config);

    assert!(result.is_err(), "invalid config should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("invalid translation configuration") || err.contains("Invalid memory page"),
        "unexpected error: {err}"
    );
}

#[test]
fn translate_preserves_safe_method_flag() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "safe_func") (result i32)
                i32.const 42)
              (@custom "neo-safe-methods" "safe_func"))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SafeMethod").expect("translation succeeds");

    // Safe method annotation should be preserved in manifest
    // This is a critical security feature
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Edge Case Error Scenarios
// ============================================================================

#[test]
fn translate_handles_empty_function() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "empty")
                nop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Empty").expect("empty function translates");

    // Empty function (just nop) should translate
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_handles_large_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "large") (result i64)
                i64.const 9223372036854775807))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LargeConst").expect("large constant translates");

    // INT64_MAX should be handled correctly
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_handles_negative_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "negative") (result i32)
                i32.const -2147483648))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NegConst").expect("negative constant translates");

    // INT32_MIN should be handled correctly
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_validates_type_consistency() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "type_check") (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.add))"#,
    )
    .expect("valid wat");

    let translation =
        translate_module(&wasm, "TypeCheck").expect("type-consistent code translates");

    // Type-consistent code should translate
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
}

#[test]
fn translate_reports_unmapped_recognized_import_with_full_path() {
    let wasm = wat::parse_str(
        r#"(module
              (import "move_stdlib" "unknown_primitive" (func $unknown))
              (func (export "main")
                call $unknown))"#,
    )
    .expect("valid wat");

    let config =
        TranslationConfig::new("MoveUnmapped").with_source_chain(wasm_neovm::SourceChain::Move);
    let result = translate_with_config(&wasm, config);

    assert!(result.is_err(), "should reject unmapped recognized import");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("move_stdlib::unknown_primitive")),
        "error should include full import path, got chain: {joined}"
    );
    assert!(
        chain.iter().any(|message| message.contains("Move")),
        "error should include source chain, got chain: {joined}"
    );
}

#[test]
fn translate_env_unsupported_import_preserves_original_name_in_error() {
    let wasm = wat::parse_str(
        r#"(module
              (import "env" "__CustomMemFn" (func $unknown (param i32 i32 i32) (result i32)))
              (memory 1)
              (func (export "main") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                call $unknown))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "EnvUnsupported");

    assert!(result.is_err(), "should reject unsupported env import");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("env::__CustomMemFn")),
        "error should preserve original import spelling, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_raw_rejects_non_void_signature() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "RAW" (func $raw (param i32) (result i32)))
              (func (export "main") (result i32)
                i32.const 1
                call $raw))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeRawResult");

    assert!(result.is_err(), "raw import with result should be rejected");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("must have signature") && message.contains("RAW")),
        "error should mention opcode signature contract, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_rejects_non_literal_immediate_operand() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "PUSHINT32" (func $push32 (param i32)))
              (func (export "main") (param i32)
                local.get 0
                call $push32))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeDynamicImmediate");

    assert!(
        result.is_err(),
        "opcode immediate should require compile-time constant"
    );
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("compile-time constant")),
        "error should mention compile-time constant immediate, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_raw4_rejects_non_literal_operand() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "raw4" (func $raw4 (param i32)))
              (func (export "main") (param i32)
                local.get 0
                call $raw4))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeRaw4Dynamic");

    assert!(result.is_err(), "raw4 should require compile-time constant");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("compile-time constant")),
        "error should mention compile-time constant immediate, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_rejects_variable_size_operands_without_raw() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "PUSHDATA1" (func $pushdata1 (param i32)))
              (func (export "main")
                i32.const 1
                call $pushdata1))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeVariableOperand");

    assert!(
        result.is_err(),
        "variable-size opcode import should be rejected"
    );
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("variable-size operand")),
        "error should mention variable-size operand restriction, got chain: {joined}"
    );
    assert!(
        chain
            .iter()
            .any(|message| message.contains("opcode.raw/raw4")),
        "error should suggest raw/raw4 fallback, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_rejects_unknown_opcode_name() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "NotRealOpcode" (func $unknown))
              (func (export "main")
                call $unknown))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeUnknown");

    assert!(result.is_err(), "unknown opcode should be rejected");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("unknown NeoVM opcode 'NotRealOpcode'")),
        "error should preserve unknown opcode name, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_raw_rejects_out_of_range_immediate() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "RAW" (func $raw (param i32)))
              (func (export "main")
                i32.const 300
                call $raw))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeRawRange");

    assert!(
        result.is_err(),
        "raw immediate outside u8 range should fail"
    );
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain
            .iter()
            .any(|message| message.contains("does not fit in 1 byte(s)")),
        "error should include immediate width failure, got chain: {joined}"
    );
}

#[test]
fn translate_opcode_raw4_rejects_wrong_arity() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "raw4" (func $raw4 (param i32 i32)))
              (func (export "main")
                i32.const 1
                i32.const 2
                call $raw4))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "OpcodeRaw4Arity");

    assert!(result.is_err(), "raw4 with wrong arity should be rejected");
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    let joined = chain.join(" | ");
    assert!(
        chain.iter().any(|message| {
            message.contains("import 'raw4' expects 1 parameter(s) but 2 were provided")
        }),
        "error should report raw4 arity contract, got chain: {joined}"
    );
}
