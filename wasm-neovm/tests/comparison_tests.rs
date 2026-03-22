// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Comprehensive comparison operation tests for WASM-NeoVM translator
// Phase 1: Critical coverage additions

use wasm_neovm::translate_module;

// ============================================================================
// i32 Comparison Tests
// ============================================================================

#[test]
fn translate_i32_eq_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "eq") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.eq))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Eq").expect("translation succeeds");

    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "should emit EQUAL for i32.eq"
    );
}

#[test]
fn translate_i32_ne_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "ne") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.ne))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Ne").expect("translation succeeds");

    let notequal = wasm_neovm::opcodes::lookup("NOTEQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&notequal),
        "should emit NOTEQUAL for i32.ne"
    );
}

#[test]
fn translate_i32_lt_s_signed_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "lt_s") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.lt_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32LtS").expect("translation succeeds");

    // Signed less-than requires LT opcode
    let lt = wasm_neovm::opcodes::lookup("LT").unwrap().byte;
    assert!(
        translation.script.contains(&lt),
        "should emit LT for i32.lt_s"
    );
}

#[test]
fn translate_i32_lt_u_unsigned_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "lt_u") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.lt_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32LtU").expect("translation succeeds");

    // Unsigned comparison needs masking or helper function
    assert!(
        !translation.script.is_empty(),
        "should generate bytecode for i32.lt_u"
    );
}

#[test]
fn translate_i32_le_s_signed_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "le_s") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.le_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32LeS").expect("translation succeeds");

    let le = wasm_neovm::opcodes::lookup("LE").unwrap().byte;
    assert!(
        translation.script.contains(&le),
        "should emit LE for i32.le_s"
    );
}

#[test]
fn translate_i32_gt_s_signed_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "gt_s") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32GtS").expect("translation succeeds");

    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    assert!(
        translation.script.contains(&gt),
        "should emit GT for i32.gt_s"
    );
}

#[test]
fn translate_i32_ge_s_signed_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "ge_s") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.ge_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32GeS").expect("translation succeeds");

    let ge = wasm_neovm::opcodes::lookup("GE").unwrap().byte;
    assert!(
        translation.script.contains(&ge),
        "should emit GE for i32.ge_s"
    );
}

// ============================================================================
// i64 Comparison Tests
// ============================================================================

#[test]
fn translate_i64_eq_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "eq") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.eq))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Eq").expect("translation succeeds");

    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "should emit EQUAL for i64.eq"
    );
}

#[test]
fn translate_i64_ne_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "ne") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.ne))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Ne").expect("translation succeeds");

    let notequal = wasm_neovm::opcodes::lookup("NOTEQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&notequal),
        "should emit NOTEQUAL for i64.ne"
    );
}

#[test]
fn translate_i64_lt_s_signed_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "lt_s") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.lt_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64LtS").expect("translation succeeds");

    let lt = wasm_neovm::opcodes::lookup("LT").unwrap().byte;
    assert!(
        translation.script.contains(&lt),
        "should emit LT for i64.lt_s"
    );
}

#[test]
fn translate_i64_gt_s_signed_comparison() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "gt_s") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.gt_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64GtS").expect("translation succeeds");

    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    assert!(
        translation.script.contains(&gt),
        "should emit GT for i64.gt_s"
    );
}

// ============================================================================
// eqz (Compare with Zero) Tests
// ============================================================================

#[test]
fn translate_i32_eqz() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "is_zero") (param i32) (result i32)
                local.get 0
                i32.eqz))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Eqz").expect("translation succeeds");

    // eqz should push 0 and compare equal
    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(translation.script.contains(&push0), "should push 0 for eqz");
    assert!(
        translation.script.contains(&equal),
        "should emit EQUAL for eqz"
    );
}

#[test]
fn translate_i64_eqz() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "is_zero") (param i64) (result i32)
                local.get 0
                i64.eqz))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Eqz").expect("translation succeeds");

    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    assert!(
        translation.script.contains(&push0),
        "should push 0 for i64.eqz"
    );
}

// ============================================================================
// Boundary Value Comparisons
// ============================================================================

#[test]
fn translate_boundary_comparison_int_min() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_min") (param i32) (result i32)
                local.get 0
                i32.const -2147483648
                i32.eq))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BoundaryMin").expect("translation succeeds");

    // Should compare with INT_MIN boundary value
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(translation.script.contains(&equal), "should emit EQUAL");
}

#[test]
fn translate_boundary_comparison_int_max() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_max") (param i32) (result i32)
                local.get 0
                i32.const 2147483647
                i32.eq))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BoundaryMax").expect("translation succeeds");

    // Should compare with INT_MAX boundary value
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(translation.script.contains(&equal), "should emit EQUAL");
}

#[test]
fn translate_boundary_comparison_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_zero") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.ne))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BoundaryZero").expect("translation succeeds");

    let notequal = wasm_neovm::opcodes::lookup("NOTEQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&notequal),
        "should emit NOTEQUAL"
    );
}

#[test]
fn translate_boundary_comparison_minus_one() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_minus_one") (param i32) (result i32)
                local.get 0
                i32.const -1
                i32.gt_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BoundaryMinusOne").expect("translation succeeds");

    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    assert!(translation.script.contains(&gt), "should emit GT");
}

// ============================================================================
// Comparison Chaining
// ============================================================================

#[test]
fn translate_comparison_chain() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "chain") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.lt_s
                local.get 1
                local.get 2
                i32.lt_s
                i32.and))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ComparisonChain").expect("translation succeeds");

    // Should emit LT twice and AND to combine
    let lt = wasm_neovm::opcodes::lookup("LT").unwrap().byte;
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    assert!(translation.script.contains(&lt), "should emit LT");
    assert!(translation.script.contains(&and), "should emit AND");
}

// ============================================================================
// Comparison Result Usage in Control Flow
// ============================================================================

#[test]
fn translate_comparison_in_if() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_if") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_s
                (if (result i32)
                  (then
                    i32.const 1)
                  (else
                    i32.const 0))))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ComparisonIf").expect("translation succeeds");

    // Should emit GT followed by conditional branching
    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    assert!(translation.script.contains(&gt), "should emit GT");
}

#[test]
fn translate_comparison_in_select() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_select") (param i32 i32) (result i32)
                i32.const 100
                i32.const 200
                local.get 0
                local.get 1
                i32.eq
                select))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ComparisonSelect").expect("translation succeeds");

    // Should emit EQUAL for comparison, then selection logic
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(translation.script.contains(&equal), "should emit EQUAL");
}

// ============================================================================
// Unsigned Comparison Edge Cases
// ============================================================================

#[test]
fn translate_unsigned_comparison_with_negatives() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "unsigned_neg") (result i32)
                i32.const -1
                i32.const 1
                i32.lt_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnsignedNegative").expect("translation succeeds");

    // -1 treated as 0xFFFFFFFF in unsigned comparison (larger than 1)
    // Should use masking or helper function for unsigned comparison
    assert!(
        !translation.script.is_empty(),
        "should generate unsigned comparison"
    );
}

#[test]
fn translate_unsigned_comparison_boundary() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "unsigned_boundary") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.ge_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnsignedBoundary").expect("translation succeeds");

    // All unsigned i32 values are >= 0, this should be optimizable
    assert!(
        !translation.script.is_empty(),
        "should generate unsigned comparison"
    );
}
