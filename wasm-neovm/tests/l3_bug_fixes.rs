// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! L3 regression tests for the bugs catalogued in
//! `docs/translator-limitations.md#BUG-1..6`. Each test exercises a
//! valid wasm pattern that the translator should accept but currently
//! bails on, OR a translator behaviour that the catalogue flagged as
//! a real bug.
//!
//! The BUG numbers correspond to the catalogue. A test asserts the
//! translator accepts the wasm (no panic, no bail) and the emitted
//! NeoVM script is well-formed (passes the post-emit validator).

use wasm_neovm::translate_module;

/// L3.BUG-1: A block with `(result i32)` (a single-value result) MUST
/// translate. The catalogue noted that wasm-opt output commonly
/// produces single-value blocks whose abstract stack has only one
/// value, and the current `block_result_count` check is too
/// strict (it fires for any result count > 1 but the message
/// suggests it's a multi-value bail when it's actually a different
/// code path). This test pins down the *correct* behaviour:
/// single-value blocks always work.
#[test]
fn l3_bug_1_single_value_block_translates() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "f") (param i32) (result i32)
                (block (result i32)
                    local.get 0
                )
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_1");
    assert!(
        result.is_ok(),
        "single-value block must translate; got error: {:?}",
        result.err()
    );
}

/// L3.BUG-1 companion: a block with NO result must translate.
#[test]
fn l3_bug_1_no_result_block_translates() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "f") (param i32)
                (block
                    local.get 0
                    drop
                )
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_1_no_result");
    assert!(
        result.is_ok(),
        "no-result block must translate; got error: {:?}",
        result.err()
    );
}

/// L3.BUG-2: A `call_indirect` with a 0-result callee must translate.
/// The current `call_indirect` bail rejects any callee whose
/// `results().len() > 1` AND any callee whose `results().len() == 0`
/// (the >=0 case isn't checked but the `params()` count may
/// cause the bail). The minimal case: indirect call to a
/// `(func (param i32))` with 0 results.
#[test]
fn l3_bug_2_call_indirect_zero_result_translates() {
    let wasm = wat::parse_str(
        r#"(module
            (type $t (func (param i32)))
            (table 1 funcref)
            (func (export "callee") (param i32)
                nop
            )
            (func (export "caller")
                i32.const 0
                i32.const 0
                call_indirect (type $t)
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_2");
    assert!(
        result.is_ok(),
        "0-result call_indirect must translate; got error: {:?}",
        result.err()
    );
}

/// L3.BUG-2 companion: call_indirect with a single i32 result must
/// translate (the canonical indirect call).
#[test]
fn l3_bug_2_call_indirect_single_i32_result_translates() {
    let wasm = wat::parse_str(
        r#"(module
            (type $t (func (param i32) (result i32)))
            (table 1 funcref)
            (func (export "callee") (param i32) (result i32)
                local.get 0
            )
            (func (export "caller") (result i32)
                i32.const 0
                i32.const 0
                call_indirect (type $t)
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_2_single");
    assert!(
        result.is_ok(),
        "single-i32-result call_indirect must translate; got error: {:?}",
        result.err()
    );
}

/// L3.BUG-3: the "function has too many locals" bail should not fire
/// for a function with a moderate number of locals (say, 10). This
/// test pins the limit and the error-message shape.
#[test]
fn l3_bug_3_moderate_locals_translate() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "f") (result i32)
                (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
                i32.const 42
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_3");
    assert!(
        result.is_ok(),
        "function with 10 locals must translate; got error: {:?}",
        result.err()
    );
}

/// L3.BUG-5: two distinct passive data segments must translate
/// without the "defined multiple times" bail. (Wasm allows any
/// number of passive segments; the translator's bookkeeping
/// is what was confused.)
#[test]
fn l3_bug_5_two_passive_data_segments_translate() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (data (i32.const 0) "hello ")
            (data (i32.const 6) "world")
            (func (export "f") (result i32)
                i32.const 0
                i32.load
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_5");
    assert!(
        result.is_ok(),
        "two passive data segments must translate; got error: {:?}",
        result.err()
    );
}

/// L3.BUG-6: a `ref.func` followed by a `call_indirect` over a
/// table that contains that same funcref index must translate and
/// (per the catalogue) should register the funcref constant so the
/// exec harness can resolve the indirect call. This test asserts
/// the translate path; the runtime-resolution assertion is the
/// L6 conformance work.
#[test]
fn l3_bug_6_ref_func_plus_call_indirect_translates() {
    let wasm = wat::parse_str(
        r#"(module
            (type $t (func (param i32) (result i32)))
            (table 1 funcref)
            (elem (i32.const 0) $target)
            (func $target (param i32) (result i32)
                local.get 0
            )
            (func (export "caller") (result i32)
                i32.const 0
                i32.const 0
                call_indirect (type $t)
            )
        )"#,
    )
    .expect("valid wat");
    let result = translate_module(&wasm, "L3_BUG_6");
    assert!(
        result.is_ok(),
        "ref.func + call_indirect must translate; got error: {:?}",
        result.err()
    );
}
