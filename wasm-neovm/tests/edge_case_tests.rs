// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_integer_overflow() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "overflow_add") (result i32)
                i32.const 2147483647
                i32.const 1
                i32.add)

              (func (export "overflow_mul") (result i32)
                i32.const 2147483647
                i32.const 2
                i32.mul)

              (func (export "overflow_sub") (result i32)
                i32.const -2147483648
                i32.const 1
                i32.sub)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "IntOverflow").expect("translation succeeds");

    let add = opcodes::lookup("ADD").unwrap().byte;
    let mul = opcodes::lookup("MUL").unwrap().byte;
    let sub = opcodes::lookup("SUB").unwrap().byte;

    assert!(translation.script.contains(&add));
    assert!(translation.script.contains(&mul));
    assert!(translation.script.contains(&sub));
}

#[test]
fn translate_division_by_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "div_by_zero") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.div_s)

              (func (export "rem_by_zero") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.rem_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivByZero").expect("translation succeeds");

    let div = opcodes::lookup("DIV").unwrap().byte;
    let mod_op = opcodes::lookup("MOD").unwrap().byte;

    assert!(translation.script.contains(&div));
    assert!(translation.script.contains(&mod_op));
}

#[test]
fn translate_null_reference_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)

              (func (export "set_null")
                i32.const 0
                ref.null func
                table.set 0)

              (func (export "check_null") (result i32)
                i32.const 0
                table.get 0
                ref.is_null
                if (result i32)
                  i32.const 1
                else
                  i32.const 0
                end)

              (func (export "assert_non_null") (result funcref)
                i32.const 0
                table.get 0
                ref.as_non_null)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "NullRef")
        .expect_err("translator should reject funcref table operations");
    let has_ref_error = err
        .chain()
        .any(|cause| cause.to_string().contains("reference type"));
    assert!(has_ref_error, "unexpected error: {err}");
}

#[test]
fn translate_stack_depth_limits() {
    let wasm = wat::parse_str(
        r#"(module
              (func $recursive (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.le_s
                if (result i32)
                  i32.const 1
                else
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $recursive
                  local.get 0
                  i32.mul
                end)

              (func (export "deep_recursion") (result i32)
                i32.const 1000
                call $recursive)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DeepRecursion").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_type_boundary_values() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "i32_limits") (result i32)
                i32.const -2147483648
                i32.const 2147483647
                i32.add)

              (func (export "i64_limits") (result i64)
                i64.const -9223372036854775808
                i64.const 9223372036854775807
                i64.add)

              (func (export "wrap_i64") (result i32)
                i64.const 4294967296
                i32.wrap_i64)

              (func (export "extend_i32") (result i64)
                i32.const -1
                i64.extend_i32_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TypeBoundaries").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_shift_overflow() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shift_32") (result i32)
                i32.const 1
                i32.const 32
                i32.shl)

              (func (export "shift_64") (result i64)
                i64.const 1
                i64.const 64
                i64.shl)

              (func (export "shift_large") (result i32)
                i32.const 1
                i32.const 100
                i32.shl)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ShiftOverflow").expect("translation succeeds");

    let shl = opcodes::lookup("SHL").unwrap().byte;
    assert!(translation.script.contains(&shl));
}

#[test]
fn translate_memory_edge_cases() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)

              (func (export "load_at_boundary") (result i32)
                i32.const 65532
                i32.load)

              (func (export "store_at_boundary")
                i32.const 65532
                i32.const 0
                i32.store)

              (func (export "load_beyond_boundary") (result i32)
                i32.const 65536
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryEdge").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    let call_s = opcodes::lookup("CALL").unwrap().byte;
    assert!(
        translation.script.contains(&call_l) || translation.script.contains(&call_s),
        "bounds checking should use helper"
    );
}

#[test]
fn translate_unreachable_trap() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "trap_immediate")
                unreachable)

              (func (export "trap_conditional") (param i32)
                local.get 0
                i32.const 0
                i32.eq
                if
                  unreachable
                end)

              (func (export "trap_after_work") (result i32)
                i32.const 42
                i32.const 1
                i32.add
                unreachable
                i32.const 2
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnreachableTrap").expect("translation succeeds");
    let abort = opcodes::lookup("ABORT").unwrap().byte;
    assert!(
        translation.script.iter().filter(|&&b| b == abort).count() >= 3,
        "expected ABORT for each unreachable"
    );
}

#[test]
fn translate_select_null_values() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)

              (func (export "select_refs") (param i32) (result funcref)
                i32.const 0
                table.get 0
                ref.null func
                local.get 0
                select)
            )"#,
    )
    .expect("valid wat");

    let err =
        translate_module(&wasm, "SelectNull").expect_err("translator should reject funcref select");
    let has_ref_error = err
        .chain()
        .any(|cause| cause.to_string().contains("reference type"));
    assert!(has_ref_error, "unexpected error: {err}");
}

#[test]
fn translate_global_boundary_access() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g1 (mut i32) (i32.const 2147483647))
              (global $g2 (mut i64) (i64.const -9223372036854775808))

              (func (export "increment_global")
                global.get $g1
                i32.const 1
                i32.add
                global.set $g1)

              (func (export "decrement_global")
                global.get $g2
                i64.const 1
                i64.sub
                global.set $g2)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalBoundary").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_local_variable_limits() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "many_locals") (result i32)
                (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)

                i32.const 0
                local.set 0
                i32.const 1
                local.set 1
                i32.const 2
                local.set 2

                local.get 0
                local.get 1
                i32.add
                local.get 2
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ManyLocals").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_empty_module() {
    let wasm = wat::parse_str(r#"(module)"#).expect("valid wat");

    let err = translate_module(&wasm, "Empty")
        .expect_err("translator should reject modules without code sections");
    assert!(err.to_string().contains("does not contain a code section"));
}

#[test]
fn translate_minimal_function() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "noop"))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Minimal").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_sign_extension_edge_cases() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend_8") (result i32)
                i32.const 255
                i32.extend8_s)

              (func (export "extend_16") (result i32)
                i32.const 65535
                i32.extend16_s)

              (func (export "extend_32") (result i64)
                i64.const 4294967295
                i64.extend32_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SignExtension").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}
