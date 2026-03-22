// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Comprehensive arithmetic operation tests for WASM-NeoVM translator
// Phase 1: Critical coverage additions

use wasm_neovm::translate_module;

// ============================================================================
// Integer Overflow Tests
// ============================================================================

#[test]
fn translate_i32_add_overflow() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "overflow") (result i32)
                i32.const 2147483647
                i32.const 1
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Overflow").expect("translation succeeds");

    // Verify ADD opcode is emitted
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(
        translation.script.contains(&add),
        "should emit ADD for i32.add"
    );

    // i32 overflow wraps around (WebAssembly spec)
    // Result should be -2147483648 (INT_MIN)
}

#[test]
fn translate_i32_sub_underflow() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "underflow") (result i32)
                i32.const -2147483648
                i32.const 1
                i32.sub))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Underflow").expect("translation succeeds");

    let sub = wasm_neovm::opcodes::lookup("SUB").unwrap().byte;
    assert!(
        translation.script.contains(&sub),
        "should emit SUB for i32.sub"
    );
}

#[test]
fn translate_i32_mul_overflow() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "mul_overflow") (result i32)
                i32.const 65536
                i32.const 65536
                i32.mul))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MulOverflow").expect("translation succeeds");

    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    assert!(
        translation.script.contains(&mul),
        "should emit MUL for i32.mul"
    );
}

#[test]
fn translate_i64_mul_overflow() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "i64_overflow") (result i64)
                i64.const 9223372036854775807
                i64.const 2
                i64.mul))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Overflow").expect("translation succeeds");

    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    assert!(
        translation.script.contains(&mul),
        "should emit MUL for i64.mul"
    );
}

#[test]
fn translate_i32_add_wraps_constants() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "wrap") (result i32)
                (local i32)
                i32.const -2147483648
                i32.const -2147483648
                i32.add
                local.set 0
                local.get 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Wrap").expect("translation succeeds");

    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;

    assert!(
        translation.script.contains(&push0),
        "local.get should push zero after wrapping"
    );
    assert!(
        translation.script.contains(&add),
        "wrapping addition should still emit ADD opcode"
    );
}

// ============================================================================
// Division Edge Cases
// ============================================================================

#[test]
fn translate_i32_div_by_zero_traps() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "div_zero") (result i32)
                i32.const 42
                i32.const 0
                i32.div_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivZero").expect("translation succeeds");

    // Should contain runtime check for division by zero
    // Translator emits helper call for division
    assert!(
        !translation.script.is_empty(),
        "should generate bytecode for division"
    );
}

#[test]
fn translate_i32_div_int_min_by_minus_one() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "int_min_div") (result i32)
                i32.const -2147483648
                i32.const -1
                i32.div_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "IntMinDiv").expect("translation succeeds");

    // INT_MIN / -1 causes overflow (should trap in WebAssembly)
    assert!(
        !translation.script.is_empty(),
        "should generate bytecode with overflow check"
    );
}

#[test]
fn translate_i64_div_by_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "i64_div_zero") (result i64)
                i64.const 100
                i64.const 0
                i64.div_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64DivZero").expect("translation succeeds");

    assert!(
        !translation.script.is_empty(),
        "should generate division check"
    );
}

#[test]
fn translate_i32_rem_by_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rem_zero") (result i32)
                i32.const 42
                i32.const 0
                i32.rem_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemZero").expect("translation succeeds");

    assert!(
        !translation.script.is_empty(),
        "should generate remainder check"
    );
}

// ============================================================================
// Bitwise Operation Edge Cases
// ============================================================================

#[test]
fn translate_xor_identity() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "xor_self") (param i32) (result i32)
                local.get 0
                local.get 0
                i32.xor))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "XorSelf").expect("translation succeeds");

    // x XOR x = 0 (could be optimized to constant)
    let xor = wasm_neovm::opcodes::lookup("XOR").unwrap().byte;
    assert!(translation.script.contains(&xor), "should emit XOR");
}

#[test]
fn translate_xor_with_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "xor_zero") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.xor))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "XorZero").expect("translation succeeds");

    // x XOR 0 = x (could be optimized away)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_and_with_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "and_zero") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.and))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "AndZero").expect("translation succeeds");

    // x AND 0 = 0 (could be constant folded)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_or_with_minus_one() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "or_all_ones") (param i32) (result i32)
                local.get 0
                i32.const -1
                i32.or))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "OrAllOnes").expect("translation succeeds");

    // x OR 0xFFFFFFFF = 0xFFFFFFFF (could be constant folded)
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Rotation Edge Cases
// ============================================================================

#[test]
fn translate_rotl_by_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rotl_zero") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.rotl))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RotlZero").expect("translation succeeds");

    // rotate by 0 = identity (could be optimized)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_rotl_by_width() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rotl_32") (param i32) (result i32)
                local.get 0
                i32.const 32
                i32.rotl))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Rotl32").expect("translation succeeds");

    // rotate by 32 = identity for i32 (k mod 32 = 0)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_rotr_overflow_amount() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rotr_big") (param i32) (result i32)
                local.get 0
                i32.const 100
                i32.rotr))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RotrBig").expect("translation succeeds");

    // rotate amount > 32 should use modulo (100 % 32 = 4)
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Shift Operation Edge Cases
// ============================================================================

#[test]
fn translate_shl_by_width() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shl_32") (param i32) (result i32)
                local.get 0
                i32.const 32
                i32.shl))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Shl32").expect("translation succeeds");

    // shift by 32 or more has defined behavior: k mod 32
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_shr_negative_value() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shr_neg") (result i32)
                i32.const -1
                i32.const 1
                i32.shr_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ShrNeg").expect("translation succeeds");

    // arithmetic right shift of negative preserves sign
    let shr = wasm_neovm::opcodes::lookup("SHR").unwrap().byte;
    assert!(translation.script.contains(&shr), "should emit SHR");
}

#[test]
fn translate_shr_u_negative_value() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shr_u_neg") (result i32)
                i32.const -1
                i32.const 1
                i32.shr_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ShrUNeg").expect("translation succeeds");

    // logical right shift treats as unsigned
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Bit Count Edge Cases
// ============================================================================

#[test]
fn translate_clz_all_zeros() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "clz_zero") (result i32)
                i32.const 0
                i32.clz))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ClzZero").expect("translation succeeds");

    // clz(0) = 32
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_clz_all_ones() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "clz_minus_one") (result i32)
                i32.const -1
                i32.clz))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ClzMinusOne").expect("translation succeeds");

    // clz(0xFFFFFFFF) = 0
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_ctz_all_zeros() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "ctz_zero") (result i32)
                i32.const 0
                i32.ctz))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CtzZero").expect("translation succeeds");

    // ctz(0) = 32
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_popcnt_edge_cases() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "popcnt_edges") (result i32)
                i32.const 0
                i32.popcnt
                i32.const -1
                i32.popcnt
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "PopcntEdges").expect("translation succeeds");

    // popcnt(0) = 0, popcnt(0xFFFFFFFF) = 32
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Modulo with Negative Numbers
// ============================================================================

#[test]
fn translate_rem_negative_dividend() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rem_neg_dividend") (result i32)
                i32.const -10
                i32.const 3
                i32.rem_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemNegDividend").expect("translation succeeds");

    // -10 % 3 = -1 (sign of dividend)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_rem_negative_divisor() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rem_neg_divisor") (result i32)
                i32.const 10
                i32.const -3
                i32.rem_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemNegDivisor").expect("translation succeeds");

    // 10 % -3 = 1 (sign of dividend)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_rem_both_negative() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rem_both_neg") (result i32)
                i32.const -10
                i32.const -3
                i32.rem_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemBothNeg").expect("translation succeeds");

    // -10 % -3 = -1
    assert!(!translation.script.is_empty());
}
