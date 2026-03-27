// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::translate_module;

#[test]
fn translate_select_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sel") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                select)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Select").expect("translation succeeds");

    let jmp_if_not_l = wasm_neovm::opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let jmp_if_not_s = wasm_neovm::opcodes::lookup("JMPIFNOT").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    let jmp_l = wasm_neovm::opcodes::lookup("JMP_L").unwrap().byte;
    let jmp_s = wasm_neovm::opcodes::lookup("JMP").unwrap().byte;
    let nip = wasm_neovm::opcodes::lookup("NIP").unwrap().byte;

    let script = &translation.script;
    // Find JMPIFNOT (long or short) followed by DROP
    let jmp_if_pos = script
        .iter()
        .enumerate()
        .find_map(|(pos, &byte)| {
            if byte == jmp_if_not_l && script.get(pos + 5) == Some(&drop) {
                Some((pos, true))
            } else if byte == jmp_if_not_s && script.get(pos + 2) == Some(&drop) {
                Some((pos, false))
            } else {
                None
            }
        })
        .expect("select emits JMPIFNOT followed by DROP");
    let (jmp_if_pos, is_long_jmpifnot) = jmp_if_pos;
    let drop_offset = if is_long_jmpifnot { 5 } else { 2 };
    assert_eq!(script[jmp_if_pos + drop_offset], drop);

    // Find JMP (long or short) followed by NIP
    let jmp_pos = script
        .iter()
        .enumerate()
        .skip(jmp_if_pos + 1)
        .find_map(|(pos, &byte)| {
            if byte == jmp_l && script.get(pos + 5) == Some(&nip) {
                Some((pos, true))
            } else if byte == jmp_s && script.get(pos + 2) == Some(&nip) {
                Some((pos, false))
            } else {
                None
            }
        })
        .expect("select emits JMP to skip else body");
    let (jmp_pos, is_long_jmp) = jmp_pos;
    assert!(jmp_pos > jmp_if_pos);
    let nip_offset = if is_long_jmp { 5 } else { 2 };
    assert_eq!(script[jmp_pos + nip_offset], nip);

    assert_eq!(script.last().copied(), Some(0x40));
}

#[test]
fn translate_ref_eq_handles_funcref() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "eq") (result i32)
                ref.null func
                ref.null func
                ref.eq))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefEq").expect("translate ref.eq");

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;

    assert_eq!(translation.script, vec![pushm1, pushm1, equal, ret]);
}

#[test]
fn translate_ref_as_non_null_traps_on_const_null() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "trap")
                ref.null func
                ref.as_non_null))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefTrap").expect("translate ref.as_non_null");

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;

    assert_eq!(translation.script, vec![pushm1, abort, ret]);
}

#[test]
fn translate_ref_as_non_null_dynamic_guard() {
    let wasm = wat::parse_str(
        r#"(module
              (table funcref (elem $f))
              (func $f)
              (func (export "guard")
                i32.const 0
                table.get 0
                ref.as_non_null
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefGuard").expect("translate guard");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    let jmpifnot_l = wasm_neovm::opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let jmpifnot_s = wasm_neovm::opcodes::lookup("JMPIFNOT").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;

    let pattern_l = [dup, pushm1, equal, jmpifnot_l];
    let pattern_s = [dup, pushm1, equal, jmpifnot_s];
    let pos = translation
        .script
        .windows(pattern_l.len())
        .position(|window| window == pattern_l || window == pattern_s)
        .expect("ref.as_non_null guard sequence present");

    let abort_pos = translation
        .script
        .iter()
        .enumerate()
        .skip(pos + pattern_l.len())
        .find(|(_, &byte)| byte == abort)
        .map(|(idx, _)| idx)
        .expect("abort present in trap path");

    assert_eq!(translation.script[abort_pos - 1], drop);
}

#[test]
fn translate_ref_null_rejects_non_func_heap_types() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "bad")
                ref.null extern
                drop))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "RefNullExtern").expect_err("externref should be rejected");
    let msg = format!("{:#}", err);
    assert!(msg.contains("ref.null"), "message: {msg}");
    assert!(
        msg.contains("docs/wasm-pipeline.md"),
        "expected docs hint: {msg}"
    );
}

#[test]
fn translate_typed_select_validates_type() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sel") (param i64 i64 i32) (result i64)
                local.get 0
                local.get 1
                local.get 2
                select (result i64))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TypedSelect").expect("translation succeeds");

    let nip = wasm_neovm::opcodes::lookup("NIP").unwrap().byte;
    assert!(translation.script.contains(&nip));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}
